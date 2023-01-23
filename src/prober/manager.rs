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
use ping::ping;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, USER_AGENT};
use reqwest::redirect::Policy as RedirectPolicy;
use reqwest::StatusCode;
use run_script::{self, ScriptOptions};

use super::replica::ReplicaURL;
use super::states::{
    ServiceStates, ServiceStatesNotifier, ServiceStatesProbe, ServiceStatesProbeNode,
    ServiceStatesProbeNodeRabbitMQ, ServiceStatesProbeNodeReplica,
    ServiceStatesProbeNodeReplicaMetrics, ServiceStatesProbeNodeReplicaMetricsRabbitMQ,
};
use super::status::Status;
use crate::config::config::{ConfigPluginsRabbitMQ, ConfigProbeServiceNodeHTTPMethod};
use crate::config::regex::Regex;
use crate::prober::manager::STORE as PROBER_STORE;
use crate::prober::mode::Mode;
use crate::APP_CONF;

const PROBE_HOLD_MILLISECONDS: u64 = 500;
const PROBE_ICMP_TIMEOUT_SECONDS: u64 = 1;

lazy_static! {
    pub static ref STORE: Arc<RwLock<Store>> = Arc::new(RwLock::new(Store {
        states: ServiceStates {
            status: Status::Healthy,
            date: None,
            probes: IndexMap::new(),
            notifier: ServiceStatesNotifier {
                reminder_backoff_counter: 1,
                reminder_ignore_until: None
            }
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

#[derive(Clone)]
struct ProbeReplicaTarget {
    pub probe_id: String,
    pub node_id: String,
    pub replica_id: String,
}

#[derive(Clone)]
struct ProbeReplicaPoll {
    pub replica_url: ReplicaURL,
    pub http_headers: HeaderMap,
    pub http_method: Option<ConfigProbeServiceNodeHTTPMethod>,
    pub http_body: Option<String>,
    pub body_match: Option<Regex>,
}

#[derive(Clone)]
struct ProbeReplicaScript {
    pub script: String,
}

#[derive(Clone)]
enum ProbeReplica {
    Poll(ProbeReplicaTarget, ProbeReplicaPoll),
    Script(ProbeReplicaTarget, ProbeReplicaScript),
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

fn map_poll_replicas() -> Vec<ProbeReplica> {
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
                        replica_list.push(ProbeReplica::Poll(
                            ProbeReplicaTarget {
                                probe_id: probe_id.to_owned(),
                                node_id: node_id.to_owned(),
                                replica_id: replica_id.to_owned(),
                            },
                            ProbeReplicaPoll {
                                replica_url: replica_url.to_owned(),
                                http_headers: node.http_headers.to_owned(),
                                http_method: node.http_method.to_owned(),
                                http_body: node.http_body.to_owned(),
                                body_match: node.http_body_healthy_match.to_owned(),
                            },
                        ));
                    }
                }
            }
        }
    }

    replica_list
}

fn map_script_replicas() -> Vec<ProbeReplica> {
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
                        replica_list.push(ProbeReplica::Script(
                            ProbeReplicaTarget {
                                probe_id: probe_id.to_owned(),
                                node_id: node_id.to_owned(),
                                replica_id: replica_id.to_owned(),
                            },
                            ProbeReplicaScript {
                                script: replica_script.to_owned(),
                            },
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
    http_headers: &HeaderMap,
    http_method: &Option<ConfigProbeServiceNodeHTTPMethod>,
    http_body: &Option<String>,
    body_match: &Option<Regex>,
) -> (Status, Option<Duration>) {
    let (mut status, mut latency, mut retry_count) = (Status::Dead, None, 0);

    while retry_count <= APP_CONF.metrics.poll_retry && status == Status::Dead {
        debug!(
            "will probe replica: {:?} with retry count: {}",
            replica_url, retry_count
        );

        thread::sleep(Duration::from_millis(PROBE_HOLD_MILLISECONDS));

        let probe_results = proceed_replica_probe_poll(
            replica_url,
            http_headers,
            http_method,
            http_body,
            body_match,
        );

        status = probe_results.0;
        latency = Some(probe_results.1);

        // Increment retry count (for next attempt)
        retry_count += 1;
    }

    (status, latency)
}

fn proceed_replica_probe_poll(
    replica_url: &ReplicaURL,
    http_headers: &HeaderMap,
    http_method: &Option<ConfigProbeServiceNodeHTTPMethod>,
    http_body: &Option<String>,
    body_match: &Option<Regex>,
) -> (Status, Duration) {
    let start_time = SystemTime::now();

    let (is_up, poll_duration) = match replica_url {
        &ReplicaURL::ICMP(ref host) => proceed_replica_probe_poll_icmp(host),
        &ReplicaURL::TCP(ref host, port) => proceed_replica_probe_poll_tcp(host, port),
        &ReplicaURL::HTTP(ref url) | &ReplicaURL::HTTPS(ref url) => {
            proceed_replica_probe_poll_http(url, http_headers, http_method, http_body, body_match)
        }
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
    http_headers: &HeaderMap,
    http_method: &Option<ConfigProbeServiceNodeHTTPMethod>,
    http_body: &Option<String>,
    body_match: &Option<Regex>,
) -> (bool, Option<Duration>) {
    // Acquire query string separator (if the URL already contains a query string, use append mode)
    let query_separator = if url.contains("?") { "&" } else { "?" };

    // Generate URL with cache buster, to bypass any upstream cache (eg. CDN cache layer)
    let url_bang = format!(
        "{}{}{}",
        url,
        query_separator,
        time::OffsetDateTime::now_utc().unix_timestamp()
    );

    // Acquire effective HTTP method to use for probe query
    let effective_http_method = http_method.as_ref().unwrap_or(if body_match.is_some() {
        &ConfigProbeServiceNodeHTTPMethod::Get
    } else {
        &ConfigProbeServiceNodeHTTPMethod::Head
    });

    // Acquire effective HTTP body to use for probe query (for POST methods only)
    let effective_http_body = http_body.as_ref().map(String::as_str).unwrap_or_default();

    // Probe target, with provided HTTP method and body (if any)
    debug!(
        "prober poll will fire for http target: {} with method: {:?} and body: '{}'",
        &url_bang, &effective_http_method, &effective_http_body
    );

    let response = match effective_http_method {
        ConfigProbeServiceNodeHTTPMethod::Head => PROBE_HTTP_CLIENT.head(&url_bang),
        ConfigProbeServiceNodeHTTPMethod::Get => PROBE_HTTP_CLIENT.get(&url_bang),
        ConfigProbeServiceNodeHTTPMethod::Post => {
            PROBE_HTTP_CLIENT
                .post(&url_bang)
                .body(reqwest::blocking::Body::from(
                    effective_http_body.to_string(),
                ))
        }
        ConfigProbeServiceNodeHTTPMethod::Put => {
            PROBE_HTTP_CLIENT
                .put(&url_bang)
                .body(reqwest::blocking::Body::from(
                    effective_http_body.to_string(),
                ))
        }
        ConfigProbeServiceNodeHTTPMethod::Patch => {
            PROBE_HTTP_CLIENT
                .patch(&url_bang)
                .body(reqwest::blocking::Body::from(
                    effective_http_body.to_string(),
                ))
        }
    }
    .headers(http_headers.to_owned())
    .send();

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
                "prober poll result was not received for http target: {} (error: {})",
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
    rabbitmq_queue: &ServiceStatesProbeNodeRabbitMQ,
) -> (bool, bool, Option<(u32, u32)>) {
    let url_queue = rabbitmq.api_url.join(&format!(
        "/api/queues/{}/{}",
        rabbitmq.virtualhost, rabbitmq_queue.queue
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
                            >= rabbitmq_queue
                                .queue_nack_healthy_below
                                .unwrap_or(rabbitmq.queue_nack_healthy_below)
                    {
                        info!(
                            "got loaded rabbitmq queue: {} (ready: {}, unacknowledged: {})",
                            rabbitmq_queue.queue,
                            response_json.messages_ready,
                            response_json.messages_unacknowledged
                        );

                        queue_loaded = true;
                    }

                    // Queue stalled?
                    if response_json.messages_ready > rabbitmq.queue_ready_dead_above
                        || response_json.messages_unacknowledged
                            > rabbitmq_queue
                                .queue_nack_dead_above
                                .unwrap_or(rabbitmq.queue_nack_dead_above)
                    {
                        info!(
                            "got stalled rabbitmq queue: {} (ready: {}, unacknowledged: {})",
                            rabbitmq_queue.queue,
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

fn dispatch_replica<'a>(probe_replica: &ProbeReplica) {
    let probe_id: &String;
    let node_id: &String;
    let replica_id: &String;

    // Acquire replica status (with optional latency)
    let (replica_status, replica_latency) = match probe_replica {
        ProbeReplica::Poll(probe_replica_target, probe_replica_poll) => {
            probe_id = &probe_replica_target.probe_id;
            node_id = &probe_replica_target.node_id;
            replica_id = &probe_replica_target.replica_id;

            proceed_replica_probe_poll_with_retry(
                &probe_replica_poll.replica_url,
                &probe_replica_poll.http_headers,
                &probe_replica_poll.http_method,
                &probe_replica_poll.http_body,
                &probe_replica_poll.body_match,
            )
        }
        ProbeReplica::Script(probe_replica_target, probe_replica_script) => {
            probe_id = &probe_replica_target.probe_id;
            node_id = &probe_replica_target.node_id;
            replica_id = &probe_replica_target.replica_id;

            proceed_replica_probe_script(&probe_replica_script.script)
        }
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

fn dispatch_replicas_in_threads(replicas: Vec<ProbeReplica>, parallelism: u16) {
    // Acquire chunk size (round to the highest unit if there is a remainder)
    let mut chunk_size = replicas.len() / parallelism as usize;

    if replicas.len() % parallelism as usize > 0 {
        chunk_size += 1;
    }

    // Anything to scan?
    if chunk_size > 0 {
        let start_time = SystemTime::now();

        // Initialize probing threads registry (this helps split the work into multiple parallel \
        //   synchronous threads, where parallelism can be increased on large Vigil setups)
        let mut prober_threads = Vec::new();

        for replicas_chunk in replicas.chunks(chunk_size) {
            // Re-map list of chunked replicas so that they can be passed to their thread
            let replicas_chunk: Vec<ProbeReplica> = replicas_chunk
                .iter()
                .map(|replica| replica.clone())
                .collect();

            // Append probing chunk into its own synchronous thread
            prober_threads.push(thread::spawn(move || {
                for probe_replica in replicas_chunk {
                    dispatch_replica(&probe_replica);
                }
            }));
        }

        let prober_threads_len = prober_threads.len();

        debug!(
            "replicas will get probed in {}/{} threads, on {} total replicas and chunk size of {}",
            prober_threads_len,
            parallelism,
            replicas.len(),
            chunk_size
        );

        // Wait for all parallel probing threads to complete
        for prober_thread in prober_threads {
            prober_thread.join().unwrap();
        }

        // Measure total probing duration
        let probing_duration = SystemTime::now()
            .duration_since(start_time)
            .unwrap_or(Duration::from_secs(0));

        info!(
            "replicas have been probed with {}/{} threads in {:?}",
            prober_threads_len, parallelism, probing_duration
        );
    }
}

fn dispatch_polls() {
    // Probe hosts
    dispatch_replicas_in_threads(map_poll_replicas(), APP_CONF.metrics.poll_parallelism);
}

fn dispatch_scripts() {
    // Run scripts
    dispatch_replicas_in_threads(map_script_replicas(), APP_CONF.metrics.script_parallelism);
}

fn dispatch_plugins_rabbitmq(
    probe_id: String,
    node_id: String,
    rabbitmq_queue: Option<ServiceStatesProbeNodeRabbitMQ>,
) {
    // RabbitMQ plugin enabled?
    if let Some(ref plugins) = APP_CONF.plugins {
        if let Some(ref rabbitmq_config) = plugins.rabbitmq {
            // Any RabbitMQ queue for node?
            if let Some(ref rabbitmq_queue_value) = rabbitmq_queue {
                // Check if RabbitMQ queue is loaded
                let mut rabbitmq_queue_load =
                    proceed_rabbitmq_queue_probe(rabbitmq_config, rabbitmq_queue_value);

                // Check once again? (the queue can be seen as loaded from check #1, but if we \
                //   check again a few milliseconds later, it will actually be empty; so not loaded)
                // Notice: this prevents false-positive 'sick' statuses.
                if rabbitmq_queue_load.0 == true {
                    if let Some(retry_delay) = rabbitmq_config.queue_loaded_retry_delay {
                        debug!(
                            "rabbitmq queue is loaded, checking once again in {}ms: {}:{} [{}]",
                            retry_delay, &probe_id, &node_id, rabbitmq_queue_value.queue
                        );

                        thread::sleep(Duration::from_millis(retry_delay));

                        // Check again if RabbitMQ queue is loaded
                        rabbitmq_queue_load =
                            proceed_rabbitmq_queue_probe(rabbitmq_config, rabbitmq_queue_value);
                    }
                }

                debug!(
                    "rabbitmq queue probe result: {}:{} [{}] => (loaded: {:?}, stalled: {:?})",
                    &probe_id,
                    &node_id,
                    rabbitmq_queue_value.queue,
                    rabbitmq_queue_load.0,
                    rabbitmq_queue_load.1
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

pub fn run_dispatch_plugins(
    probe_id: &str,
    node_id: &str,
    rabbitmq_queue: Option<ServiceStatesProbeNodeRabbitMQ>,
) {
    // Check target RabbitMQ queue?
    if let Some(ref plugins) = APP_CONF.plugins {
        if plugins.rabbitmq.is_some() {
            let self_probe_id = probe_id.to_owned();
            let self_node_id = node_id.to_owned();

            thread::spawn(move || {
                dispatch_plugins_rabbitmq(self_probe_id, self_node_id, rabbitmq_queue)
            });
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
                http_headers: node.http_headers.to_owned(),
                http_method: node.http_method.to_owned(),
                http_body: node.http_body.to_owned(),
                http_body_healthy_match: node.http_body_healthy_match.to_owned(),
                reveal_replica_name: node.reveal_replica_name,
                rabbitmq: node.rabbitmq_queue.as_ref().map(|queue| {
                    ServiceStatesProbeNodeRabbitMQ {
                        queue: queue.to_owned(),
                        queue_nack_healthy_below: node.rabbitmq_queue_nack_healthy_below,
                        queue_nack_dead_above: node.rabbitmq_queue_nack_dead_above,
                    }
                }),
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
