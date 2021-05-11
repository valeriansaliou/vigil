// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::cmp::min;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::{Duration, SystemTime};
use time;

use indexmap::IndexMap;
use log::{debug, error, info, warn};
use ping::ping;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, USER_AGENT};
use reqwest::redirect::Policy as RedirectPolicy;
use reqwest::StatusCode;
use run_script::{self, ScriptOptions};
use serde_derive::Deserialize;

use super::replica::ReplicaURL;
use super::states::{
    ServiceStates, ServiceStatesProbe, ServiceStatesProbeNode, ServiceStatesProbeNodeReplica,
    ServiceStatesProbeNodeReplicaMetrics, ServiceStatesProbeNodeReplicaMetricsRabbitMQ,
};
use super::status::Status;
use crate::config::config::ConfigPluginsRabbitMQ;
use crate::config::regex::Regex;
use crate::prober::manager::STORE as PROBER_STORE;
use crate::prober::mode::Mode;
use crate::APP_CONF;

const PROBE_HOLD_MILLISECONDS: u64 = 250;
const PROBE_ICMP_TIMEOUT_SECONDS: u64 = 1;

lazy_static::lazy_static! {
    pub static ref STORE: Arc<RwLock<Store>> = Arc::new(RwLock::new(Store {
        states: ServiceStates {
            status: Status::Healthy,
            date: None,
            probes: IndexMap::new(),
        },
        notified: None,
    }));
    static ref PROBE_HTTP_CLIENT: Client = Client::builder()
        .timeout(Duration::from_secs(APP_CONF.metrics.poll_delay_dead))
        .gzip(false)
        .redirect(RedirectPolicy::none())
        .default_headers(make_default_headers())
        .build()
        .unwrap();
}

#[derive(Deserialize)]
struct RabbitMQAPIQueueResponse {
    messages_ready: u32,
    messages_unacknowledged: u32,
}

pub struct Store {
    pub states: ServiceStates,
    pub notified: Option<SystemTime>,
}

enum DispatchMode<'a> {
    Poll(&'a ReplicaURL, &'a Option<Regex>),
    Script(&'a String),
}

fn make_default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.insert(
        USER_AGENT,
        format!("vigil (+{})", APP_CONF.branding.page_url.as_str())
            .parse()
            .unwrap(),
    );

    headers
}

fn map_poll_replicas() -> Vec<(String, String, String, ReplicaURL, Option<Regex>)> {
    let mut replica_list = Vec::new();

    // Acquire states
    let states = &PROBER_STORE.read().unwrap().states;

    // Map replica URLs to be probed
    for (probe_id, probe) in states.probes.iter() {
        for (node_id, node) in probe.nodes.iter() {
            if node.mode == Mode::Poll {
                for (replica_id, replica) in node.replicas.iter() {
                    if let Some(ref replica_url) = replica.url {
                        // Clone values to scan; this ensure the write lock is not held while \
                        //   the replica scan is performed. As this whole operation can take time, \
                        //   it could lock all the pipelines depending on the shared store data \
                        //   (eg. the reporter HTTP API).
                        replica_list.push((
                            probe_id.to_owned(),
                            node_id.to_owned(),
                            replica_id.to_owned(),
                            replica_url.to_owned(),
                            node.http_body_healthy_match.to_owned(),
                        ));
                    }
                }
            }
        }
    }

    replica_list
}

fn map_script_replicas() -> Vec<(String, String, String, String)> {
    let mut replica_list = Vec::new();

    // Acquire states
    let states = &PROBER_STORE.read().unwrap().states;

    // Map scripts to be probed
    for (probe_id, probe) in states.probes.iter() {
        for (node_id, node) in probe.nodes.iter() {
            if node.mode == Mode::Script {
                for (replica_id, replica) in node.replicas.iter() {
                    if let Some(ref replica_script) = replica.script {
                        // Clone values to scan; this ensure the write lock is not held while \
                        //   the script execution is performed. Same as in `map_poll_replicas()`.
                        replica_list.push((
                            probe_id.to_owned(),
                            node_id.to_owned(),
                            replica_id.to_owned(),
                            replica_script.to_owned(),
                        ));
                    }
                }
            }
        }
    }

    replica_list
}

fn proceed_replica_probe_poll_with_retry(
    replica_url: &ReplicaURL,
    body_match: &Option<Regex>,
) -> (Status, Option<Duration>) {
    let (mut status, mut latency, mut retry_count) = (Status::Dead, None, 0);

    while retry_count < APP_CONF.metrics.poll_retry && status == Status::Dead {
        retry_count += 1;

        debug!(
            "will probe replica: {:?} with retry count: {}",
            replica_url, retry_count
        );

        thread::sleep(Duration::from_millis(PROBE_HOLD_MILLISECONDS));

        let probe_results = proceed_replica_probe_poll(replica_url, body_match);

        status = probe_results.0;
        latency = Some(probe_results.1);
    }

    (status, latency)
}

fn proceed_replica_probe_poll(
    replica_url: &ReplicaURL,
    body_match: &Option<Regex>,
) -> (Status, Duration) {
    let start_time = SystemTime::now();

    let (is_up, poll_duration) = match replica_url {
        &ReplicaURL::ICMP(ref host) => proceed_replica_probe_poll_icmp(host),
        &ReplicaURL::TCP(ref host, port) => proceed_replica_probe_poll_tcp(host, port),
        &ReplicaURL::HTTP(ref url) => proceed_replica_probe_poll_http(url, body_match),
        &ReplicaURL::HTTPS(ref url) => proceed_replica_probe_poll_http(url, body_match),
    };

    let duration_latency = match poll_duration {
        Some(poll_duration) => poll_duration,
        None => SystemTime::now()
            .duration_since(start_time)
            .unwrap_or(Duration::from_secs(0)),
    };

    if is_up == true {
        // Probe reports as sick?
        if duration_latency >= Duration::from_secs(APP_CONF.metrics.poll_delay_sick) {
            return (Status::Sick, duration_latency);
        }

        (Status::Healthy, duration_latency)
    } else {
        (Status::Dead, duration_latency)
    }
}

fn proceed_replica_probe_poll_icmp(host: &str) -> (bool, Option<Duration>) {
    // Notice: a dummy port of value '0' is set here, so that we can resolve the host to an actual \
    //   IP address using the standard library, which avoids depending on an additional library.
    let address_results = (host, 0).to_socket_addrs();

    // Storage variable for the maximum round-trip-time found for received ping responses
    let mut maximum_rtt = None;

    match address_results {
        Ok(address) => {
            // Notice: the ICMP probe checker is a bit special, in the sense that it checks all \
            //   resolved addresses. As we check for an host health at the IP level (ie. not at \
            //   the application layer level), checking only the first host in the list is not \
            //   sufficient for the whole replica group to be up. This can be used as an handy way \
            //   to check for the health of a group of IP hosts, configured in a single DNS record.
            let address_values: Vec<SocketAddr> = address.collect();

            if !address_values.is_empty() {
                debug!(
                    "prober poll will fire for icmp host: {} ({} targets)",
                    host,
                    address_values.len()
                );

                // As ICMP pings require a lower-than-usual timeout, an hard-coded ICMP \
                //   timeout value is used by default, though the configured dead delay value \
                //   is preferred in the event it is lower than the hard-coded value (unlikely \
                //   though possible in some setups).
                let pinger_timeout = Duration::from_secs(min(
                    PROBE_ICMP_TIMEOUT_SECONDS,
                    APP_CONF.metrics.poll_delay_dead,
                ));

                // Probe all returned addresses (sequentially)
                for address_value in &address_values {
                    let address_ip = address_value.ip();

                    debug!(
                        "prober poll will send icmp ping to target: {} from host: {}",
                        address_ip, host
                    );

                    // Acquire ping start time (used for RTT calculation)
                    let ping_start_time = SystemTime::now();

                    // Ping target IP address
                    match ping(address_ip, Some(pinger_timeout), None, None, None, None) {
                        Ok(_) => {
                            debug!(
                                "got prober poll response for icmp target: {} from host: {}",
                                address_ip, host
                            );

                            // Process ping RTT
                            let ping_rtt = SystemTime::now()
                                .duration_since(ping_start_time)
                                .unwrap_or(Duration::from_secs(0));

                            // Do not return (consider address as reachable)
                            // Notice: update maximum observed round-trip-time, if higher than \
                            //   last highest observed.
                            maximum_rtt = match maximum_rtt {
                                Some(maximum_rtt) => {
                                    if ping_rtt > maximum_rtt {
                                        Some(ping_rtt)
                                    } else {
                                        Some(maximum_rtt)
                                    }
                                }
                                None => Some(ping_rtt),
                            };
                        }
                        Err(err) => {
                            debug!(
                                "prober poll error for icmp target: {} from host: {} (error: {})",
                                address_ip, host, err
                            );

                            // Consider ICMP errors as a failure
                            return (false, None);
                        }
                    }
                }
            } else {
                debug!(
                    "prober poll did not resolve any address for icmp replica: {}",
                    host
                );

                // Consider empty as a failure
                return (false, None);
            }
        }
        Err(err) => {
            error!(
                "prober poll address for icmp replica is invalid: {} (error: {})",
                host, err
            );

            // Consider invalid URL as a failure
            return (false, None);
        }
    };

    // If there was no early return, consider all the hosts as reachable for replica
    (true, maximum_rtt)
}

fn proceed_replica_probe_poll_tcp(host: &str, port: u16) -> (bool, Option<Duration>) {
    let address_results = (host, port).to_socket_addrs();

    match address_results {
        Ok(mut address) => {
            if let Some(address_value) = address.next() {
                debug!("prober poll will fire for tcp target: {}", address_value);

                return match TcpStream::connect_timeout(
                    &address_value,
                    Duration::from_secs(APP_CONF.metrics.poll_delay_dead),
                ) {
                    Ok(_) => {
                        debug!("prober poll success for tcp target: {}", address_value);

                        (true, None)
                    }
                    Err(err) => {
                        debug!(
                            "prober poll error for tcp target: {} (error: {})",
                            address_value, err
                        );

                        (false, None)
                    }
                };
            } else {
                debug!(
                    "prober poll did not resolve any address for tcp replica: {}:{}",
                    host, port
                );
            }
        }
        Err(err) => {
            error!(
                "prober poll address for tcp replica is invalid: {}:{} (error: {})",
                host, port, err
            );
        }
    };

    (false, None)
}

fn proceed_replica_probe_poll_http(
    url: &str,
    body_match: &Option<Regex>,
) -> (bool, Option<Duration>) {
    // Acquire query string separator (if the URL already contains a query string, use append mode)
    let query_separator = if url.contains("?") { "&" } else { "?" };

    // Generate URL with cache buster, to bypass any upstream cache (eg. CDN cache layer)
    let url_bang = format!(
        "{}{}{}",
        url,
        query_separator,
        time::now().to_timespec().sec
    );

    debug!("prober poll will fire for http target: {}", &url_bang);

    let response = if body_match.is_some() {
        PROBE_HTTP_CLIENT.get(&url_bang).send()
    } else {
        PROBE_HTTP_CLIENT.head(&url_bang).send()
    };

    match response {
        Ok(response_inner) => {
            let status_code = response_inner.status().as_u16();

            debug!(
                "prober poll result received for http target: {} with status: {}",
                &url_bang, status_code
            );

            // Consider as UP?
            if status_code >= APP_CONF.metrics.poll_http_status_healthy_above
                && status_code < APP_CONF.metrics.poll_http_status_healthy_below
            {
                // Check response body for match? (if configured)
                if let &Some(ref body_match_regex) = body_match {
                    if let Ok(text) = response_inner.text() {
                        debug!(
                        "checking prober poll response text for http target: {} for any match: {}",
                        &url_bang, &text
                    );

                        // Doesnt match? Consider as DOWN.
                        if body_match_regex.is_match(&text) == false {
                            return (false, None);
                        }
                    } else {
                        debug!(
                            "could not unpack response text for http target: {}",
                            &url_bang
                        );

                        // Consider as DOWN (the response text could not be checked)
                        return (false, None);
                    }
                }

                return (true, None);
            }
        }
        Err(err) => {
            debug!(
                "prober poll result was not received for http target: {}, {}",
                &url_bang, err
            );
        }
    }

    // Consider as DOWN.
    (false, None)
}

fn proceed_replica_probe_script(script: &String) -> (Status, Option<Duration>) {
    let start_time = SystemTime::now();

    let status = match run_script::run(script, &Vec::new(), &ScriptOptions::new()) {
        Ok((code, _, _)) => {
            debug!(
                "prober script execution succeeded with return code: {}",
                code
            );

            // Return code '0' goes for 'healthy', '1' goes for 'sick'; any other code is 'dead'
            match code {
                0 => Status::Healthy,
                1 => Status::Sick,
                _ => Status::Dead,
            }
        }
        Err(err) => {
            error!("prober script execution failed with error: {}", err);

            Status::Dead
        }
    };

    (status, SystemTime::now().duration_since(start_time).ok())
}

fn proceed_rabbitmq_queue_probe(
    rabbitmq: &ConfigPluginsRabbitMQ,
    rabbitmq_queue: &str,
) -> (bool, bool, Option<(u32, u32)>) {
    let url_queue = rabbitmq.api_url.join(&format!(
        "/api/queues/{}/{}",
        rabbitmq.virtualhost, rabbitmq_queue
    ));

    if let Ok(url_queue_value) = url_queue {
        let url_queue_string = url_queue_value.as_str();

        debug!(
            "prober poll will fire for rabbitmq queue at url: {}",
            url_queue_string
        );

        let response = PROBE_HTTP_CLIENT
            .get(url_queue_string)
            .basic_auth(
                rabbitmq.auth_username.to_owned(),
                Some(rabbitmq.auth_password.to_owned()),
            )
            .send();

        if let Ok(response_inner) = response {
            let status = response_inner.status();

            debug!(
                "prober poll on rabbitmq queue result received for url: {} with status: {}",
                url_queue_string,
                status.as_u16()
            );

            // Check JSON result?
            if status == StatusCode::OK {
                if let Ok(response_json) = response_inner.json::<RabbitMQAPIQueueResponse>() {
                    let (mut queue_loaded, mut queue_stalled) = (false, false);

                    let queue_counts = Some((
                        response_json.messages_ready,
                        response_json.messages_unacknowledged,
                    ));

                    // Queue loaded?
                    if response_json.messages_ready >= rabbitmq.queue_ready_healthy_below
                        || response_json.messages_unacknowledged
                            >= rabbitmq.queue_nack_healthy_below
                    {
                        info!(
                            "got loaded rabbitmq queue: {} (ready: {}, unacknowledged: {})",
                            rabbitmq_queue,
                            response_json.messages_ready,
                            response_json.messages_unacknowledged
                        );

                        queue_loaded = true;
                    }

                    // Queue stalled?
                    if response_json.messages_ready > rabbitmq.queue_ready_dead_above
                        || response_json.messages_unacknowledged > rabbitmq.queue_nack_dead_above
                    {
                        info!(
                            "got stalled rabbitmq queue: {} (ready: {}, unacknowledged: {})",
                            rabbitmq_queue,
                            response_json.messages_ready,
                            response_json.messages_unacknowledged
                        );

                        queue_stalled = true;
                    }

                    return (queue_loaded, queue_stalled, queue_counts);
                }
            } else {
                warn!(
                    "rabbitmq api replied with an invalid status code: {}",
                    status.as_u16()
                );
            }
        } else {
            warn!("rabbitmq api request failed");
        }
    }

    (false, false, None)
}

fn dispatch_replica<'a>(mode: DispatchMode<'a>, probe_id: &str, node_id: &str, replica_id: &str) {
    // Acquire replica status (with optional latency)
    let (replica_status, replica_latency) = match mode {
        DispatchMode::Poll(replica_url, body_match) => {
            proceed_replica_probe_poll_with_retry(replica_url, body_match)
        }
        DispatchMode::Script(script) => proceed_replica_probe_script(script),
    };

    debug!(
        "replica probe result: {}:{}:{} => {:?}",
        probe_id, node_id, replica_id, replica_status
    );

    // Update replica status (write-lock the store)
    {
        let mut store = STORE.write().unwrap();

        if let Some(ref mut probe) = store.states.probes.get_mut(probe_id) {
            if let Some(ref mut node) = probe.nodes.get_mut(node_id) {
                if let Some(ref mut replica) = node.replicas.get_mut(replica_id) {
                    replica.status = replica_status;

                    replica.metrics.latency =
                        replica_latency.map(|duration| duration.as_millis() as u64);
                }
            }
        }
    }
}

fn dispatch_polls() {
    // Probe hosts
    for probe_replica in map_poll_replicas() {
        dispatch_replica(
            DispatchMode::Poll(&probe_replica.3, &probe_replica.4),
            &probe_replica.0,
            &probe_replica.1,
            &probe_replica.2,
        );
    }
}

fn dispatch_scripts() {
    // Run scripts
    for probe_replica in map_script_replicas() {
        dispatch_replica(
            DispatchMode::Script(&probe_replica.3),
            &probe_replica.0,
            &probe_replica.1,
            &probe_replica.2,
        );
    }
}

fn dispatch_plugins_rabbitmq(probe_id: String, node_id: String, queue: Option<String>) {
    // RabbitMQ plugin enabled?
    if let Some(ref plugins) = APP_CONF.plugins {
        if let Some(ref rabbitmq) = plugins.rabbitmq {
            // Any queue for node?
            if let Some(ref queue_value) = queue {
                // Check if RabbitMQ queue is loaded
                let mut rabbitmq_queue_load = proceed_rabbitmq_queue_probe(rabbitmq, queue_value);

                // Check once again? (the queue can be seen as loaded from check #1, but if we \
                //   check again a few milliseconds later, it will actually be empty; so not loaded)
                // Notice: this prevents false-positive 'sick' statuses.
                if rabbitmq_queue_load.0 == true {
                    if let Some(retry_delay) = rabbitmq.queue_loaded_retry_delay {
                        debug!(
                            "rabbitmq queue is loaded, checking once again in {}ms: {}:{} [{}]",
                            retry_delay, &probe_id, &node_id, queue_value
                        );

                        thread::sleep(Duration::from_millis(retry_delay));

                        // Check again if RabbitMQ queue is loaded
                        rabbitmq_queue_load = proceed_rabbitmq_queue_probe(rabbitmq, queue_value);
                    }
                }

                debug!(
                    "rabbitmq queue probe result: {}:{} [{}] => (loaded: {:?}, stalled: {:?})",
                    &probe_id, &node_id, queue_value, rabbitmq_queue_load.0, rabbitmq_queue_load.1
                );

                // Update replica status (write-lock the store)
                {
                    let mut store = STORE.write().unwrap();

                    if let Some(ref mut probe) = store.states.probes.get_mut(&probe_id) {
                        if let Some(ref mut node) = probe.nodes.get_mut(&node_id) {
                            for (_, replica) in node.replicas.iter_mut() {
                                if let Some(ref mut replica_load) = replica.load {
                                    replica_load.queue.loaded = rabbitmq_queue_load.0;
                                    replica_load.queue.stalled = rabbitmq_queue_load.1;
                                }

                                // Store RabbitMQ metrics
                                if let Some((queue_ready, queue_nack)) = rabbitmq_queue_load.2 {
                                    replica.metrics.rabbitmq =
                                        Some(ServiceStatesProbeNodeReplicaMetricsRabbitMQ {
                                            queue_ready: queue_ready,
                                            queue_nack: queue_nack,
                                        });
                                } else {
                                    replica.metrics.rabbitmq = None
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn run_dispatch_plugins(probe_id: &str, node_id: &str, queue: Option<String>) {
    // Check target RabbitMQ queue?
    if let Some(ref plugins) = APP_CONF.plugins {
        if plugins.rabbitmq.is_some() {
            let self_probe_id = probe_id.to_owned();
            let self_node_id = node_id.to_owned();

            thread::spawn(move || dispatch_plugins_rabbitmq(self_probe_id, self_node_id, queue));
        }
    }
}

pub fn initialize_store() {
    // Copy monitored hosts in store (refactor the data structure)
    let mut store = STORE.write().unwrap();

    for service in &APP_CONF.probe.service {
        let mut probe = ServiceStatesProbe {
            id: service.id.to_owned(),
            label: service.label.to_owned(),
            status: Status::Healthy,
            nodes: IndexMap::new(),
        };

        debug!("prober store: got service {}", service.id);

        for node in &service.node {
            debug!("prober store: got node {}:{}", service.id, node.id);

            let mut probe_node = ServiceStatesProbeNode {
                status: Status::Healthy,
                label: node.label.to_owned(),
                mode: node.mode.to_owned(),
                replicas: IndexMap::new(),
                http_body_healthy_match: node.http_body_healthy_match.to_owned(),
                rabbitmq_queue: node.rabbitmq_queue.to_owned(),
            };

            // Node with replicas? (might be a poll node)
            if let Some(ref replicas) = node.replicas {
                if node.mode != Mode::Poll {
                    panic!("non-poll node cannot have replicas");
                }

                for replica in replicas {
                    debug!(
                        "prober store: got replica {}:{}:{}",
                        service.id, node.id, replica
                    );

                    let replica_url = ReplicaURL::parse_from(replica).expect("invalid replica url");

                    probe_node.replicas.insert(
                        replica.to_string(),
                        ServiceStatesProbeNodeReplica {
                            status: Status::Healthy,
                            url: Some(replica_url),
                            script: None,
                            metrics: ServiceStatesProbeNodeReplicaMetrics::default(),
                            load: None,
                            report: None,
                        },
                    );
                }
            }

            // Node with scripts? (might be a script node)
            if let Some(ref scripts) = node.scripts {
                if node.mode != Mode::Script {
                    panic!("non-script node cannot have scripts");
                }

                for (index, script) in scripts.iter().enumerate() {
                    debug!(
                        "prober store: got script {}:{}:#{}",
                        service.id, node.id, index
                    );

                    probe_node.replicas.insert(
                        index.to_string(),
                        ServiceStatesProbeNodeReplica {
                            status: Status::Healthy,
                            url: None,
                            script: Some(script.to_owned()),
                            metrics: ServiceStatesProbeNodeReplicaMetrics::default(),
                            load: None,
                            report: None,
                        },
                    );
                }
            }

            probe.nodes.insert(node.id.to_owned(), probe_node);
        }

        store.states.probes.insert(service.id.to_owned(), probe);
    }

    info!("initialized prober store");
}

pub fn run_poll() {
    loop {
        debug!("running a poll probe operation...");

        dispatch_polls();

        info!("ran poll probe operation");

        // Hold for next aggregate run
        thread::sleep(Duration::from_secs(APP_CONF.metrics.poll_interval));
    }
}

pub fn run_script() {
    loop {
        debug!("running a script probe operation...");

        dispatch_scripts();

        info!("ran script probe operation");

        // Hold for next aggregate run
        thread::sleep(Duration::from_secs(APP_CONF.metrics.script_interval));
    }
}
