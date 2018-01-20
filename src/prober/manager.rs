// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::RwLock;
use std::sync::Arc;
use std::thread;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{SystemTime, Duration};
use time;

use reqwest::{Client, RedirectPolicy};
use reqwest::header::{Headers, UserAgent};
use ordermap::OrderMap;

use prober::manager::STORE as PROBER_STORE;
use prober::mode::Mode;
use super::states::{ServiceStates, ServiceStatesProbe, ServiceStatesProbeNode,
                    ServiceStatesProbeNodeReplica};
use super::replica::ReplicaURL;
use super::status::Status;
use APP_CONF;

const PROBE_HOLD_MILLISECONDS: u64 = 250;

lazy_static! {
    pub static ref STORE: Arc<RwLock<Store>> = Arc::new(RwLock::new(Store {
        states: ServiceStates {
            status: Status::Healthy,
            date: None,
            probes: OrderMap::new(),
        }
    }));

    static ref PROBE_HTTP_CLIENT: Client = Client::builder()
        .timeout(Duration::from_secs(APP_CONF.metrics.poll_delay_dead))
        .gzip(false)
        .redirect(RedirectPolicy::none())
        .enable_hostname_verification()
        .default_headers(make_default_headers())
        .build()
        .unwrap();
}

pub struct Store {
    pub states: ServiceStates,
}

fn make_default_headers() -> Headers {
    let mut headers = Headers::new();

    headers.set(UserAgent::new(
        format!("vigil (+{})", APP_CONF.branding.page_url.as_str()),
    ));

    headers
}

fn map_poll_replicas() -> Vec<(String, String, String, ReplicaURL)> {
    let mut replica_list = Vec::new();

    // Acquire states
    let states = &PROBER_STORE.read().unwrap().states;

    // Map hosts to be probed
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
                        ));
                    }
                }
            }
        }
    }

    replica_list
}

fn proceed_replica_probe_with_retry(replica_url: &ReplicaURL) -> Status {
    let mut status = Status::Dead;
    let mut retry_count = 0;

    while retry_count < APP_CONF.metrics.poll_retry && status == Status::Dead {
        retry_count += 1;

        debug!(
            "will probe replica: {:?} with retry count: {}",
            replica_url,
            retry_count
        );

        thread::sleep(Duration::from_millis(PROBE_HOLD_MILLISECONDS));

        status = proceed_replica_probe(replica_url);
    }

    status
}

fn proceed_replica_probe(replica_url: &ReplicaURL) -> Status {
    let start_time = SystemTime::now();

    let is_up = match replica_url {
        &ReplicaURL::TCP(ref host, port) => proceed_replica_probe_tcp(host, port),
        &ReplicaURL::HTTP(ref url) => proceed_replica_probe_http(url),
        &ReplicaURL::HTTPS(ref url) => proceed_replica_probe_http(url),
    };

    if is_up == true {
        // Probe reports as sick?
        if let Ok(duration_since) = SystemTime::now().duration_since(start_time) {
            if duration_since >= Duration::from_secs(APP_CONF.metrics.poll_delay_sick) {
                return Status::Sick;
            }
        }

        Status::Healthy
    } else {
        Status::Dead
    }
}

fn proceed_replica_probe_tcp(host: &str, port: u16) -> bool {
    let address_results = (host, port).to_socket_addrs();

    if let Ok(mut address) = address_results {
        if let Some(address_value) = address.next() {
            debug!("prober poll will fire for tcp target: {}", address_value);

            return match TcpStream::connect_timeout(
                &address_value,
                Duration::from_secs(APP_CONF.metrics.poll_delay_dead),
            ) {
                Ok(_) => true,
                Err(_) => false,
            };
        }
    }

    false
}

fn proceed_replica_probe_http(url: &str) -> bool {
    let url_bang = format!("{}?{}", url, time::now().to_timespec().sec);

    debug!("prober poll will fire for http target: {}", &url_bang);

    let response = PROBE_HTTP_CLIENT.head(&url_bang).send();

    if let Ok(response_inner) = response {
        let status_code = response_inner.status().as_u16();

        debug!(
            "prober poll result received for url: {} with status: {}",
            &url_bang,
            status_code
        );

        // Consider as UP?
        if status_code >= APP_CONF.metrics.poll_http_status_healthy_above &&
            status_code < APP_CONF.metrics.poll_http_status_healthy_below
        {
            return true;
        }
    }

    // Consider as DOWN.
    false
}

fn dispatch_polls() {
    // Probe hosts
    for probe_replica in map_poll_replicas() {
        let replica_status = proceed_replica_probe_with_retry(&probe_replica.3);

        debug!(
            "probe result: {}:{}:{} => {:?}",
            &probe_replica.0,
            &probe_replica.1,
            &probe_replica.2,
            replica_status
        );

        // Update replica status (write-lock the store)
        {
            let mut store = STORE.write().unwrap();

            if let Some(ref mut probe) = store.states.probes.get_mut(&probe_replica.0) {
                if let Some(ref mut node) = probe.nodes.get_mut(&probe_replica.1) {
                    if let Some(ref mut replica) = node.replicas.get_mut(&probe_replica.2) {
                        replica.status = replica_status;
                    }
                }
            }
        }
    }
}

pub fn initialize_store() {
    // Copy monitored hosts in store (refactor the data structure)
    let mut store = STORE.write().unwrap();

    for service in &APP_CONF.probe.service {
        let mut probe = ServiceStatesProbe {
            status: Status::Healthy,
            label: service.label.to_owned(),
            nodes: OrderMap::new(),
        };

        debug!("prober store: got service {}", service.id);

        for node in &service.node {
            debug!("prober store: got node {}:{}", service.id, node.id);

            let mut probe_node = ServiceStatesProbeNode {
                status: Status::Healthy,
                label: node.label.to_owned(),
                mode: node.mode.to_owned(),
                replicas: OrderMap::new(),
            };

            if let Some(ref replicas) = node.replicas {
                if node.mode != Mode::Poll {
                    panic!("non-poll node cannot have replicas");
                }

                for replica in replicas {
                    debug!(
                        "prober store: got replica {}:{}:{}",
                        service.id,
                        node.id,
                        replica
                    );

                    let replica_url = ReplicaURL::parse_from(replica).expect("invalid replica url");

                    probe_node.replicas.insert(
                        replica.to_string(),
                        ServiceStatesProbeNodeReplica {
                            status: Status::Healthy,
                            url: Some(replica_url),
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

pub fn run() {
    loop {
        debug!("running a probe operation...");

        dispatch_polls();

        info!("ran probe operation");

        // Hold for next aggregate run
        thread::sleep(Duration::from_secs(APP_CONF.metrics.poll_interval));
    }
}
