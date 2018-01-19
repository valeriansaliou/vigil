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

use super::states::{
    ServiceStates,
    ServiceStatesProbe,
    ServiceStatesProbeNode,
    ServiceStatesProbeNodeReplica
};
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
                for replica in replicas {
                    debug!("prober store: got replica {}:{}:{}", service.id, node.id, replica);

                    probe_node.replicas.insert(replica.to_string(), ServiceStatesProbeNodeReplica {
                        status: Status::Healthy,
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
    // TODO
    thread::sleep(Duration::from_secs(120));

    panic!("not implemented");
}
