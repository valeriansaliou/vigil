// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::RwLock;
use std::sync::Arc;
use std::time::Duration;
use std::thread;

use ordermap::OrderMap;

use prober::manager::{STORE as PROBER_STORE};
use prober::mode::Mode;
use super::states::{
    ServiceStates,
    ServiceStatesProbe,
    ServiceStatesProbeNode,
    ServiceStatesProbeNodeReplica
};
use super::replica::ReplicaURL;
use super::status::Status;
use APP_CONF;

lazy_static! {
    pub static ref STORE: Arc<RwLock<Store>> = Arc::new(RwLock::new(Store {
        states: ServiceStates {
            status: Status::Healthy,
            date: None,
            probes: OrderMap::new(),
        }
    }));
}

pub struct Store {
    pub states: ServiceStates,
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
                            replica_url.to_owned()
                        ));
                    }
                }
            }
        }
    }

    replica_list
}

fn proceed_replica_probe(replica_url: &ReplicaURL) -> Status {
    match replica_url {
        &ReplicaURL::TCP(ref host, port) => proceed_replica_probe_tcp(host, port),
        &ReplicaURL::HTTP(ref host, port) => proceed_replica_probe_http(host, port),
    }
}

fn proceed_replica_probe_tcp(host: &str, port: u16) -> Status {
    // TODO
    Status::Sick
}

fn proceed_replica_probe_http(host: &str, port: u16) -> Status {
    // TODO
    Status::Sick
}

fn dispatch_polls() {
    // Probe hosts
    for probe_replica in map_poll_replicas() {
        let replica_status = proceed_replica_probe(&probe_replica.3);

        debug!(
            "probe result: {}:{}:{} => {:?}", &probe_replica.0, &probe_replica.1, &probe_replica.2,
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
            nodes: OrderMap::new()
        };

        debug!("prober store: got service {}", service.id);

        for node in &service.node {
            debug!("prober store: got node {}:{}", service.id, node.id);

            let mut probe_node = ServiceStatesProbeNode {
                status: Status::Healthy,
                label: node.label.to_owned(),
                mode: node.mode.to_owned(),
                replicas: OrderMap::new()
            };

            if let Some(ref replicas) = node.replicas {
                if node.mode != Mode::Poll {
                    panic!("non-poll node cannot have replicas");
                }

                for replica in replicas {
                    debug!("prober store: got replica {}:{}:{}", service.id, node.id, replica);

                    let replica_url = ReplicaURL::parse_from(replica).expect("invalid replica url");

                    probe_node.replicas.insert(replica.to_string(), ServiceStatesProbeNodeReplica {
                        status: Status::Healthy,
                        url: Some(replica_url),
                        load: None,
                        report: None,
                    });
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
