// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::{SystemTime, Duration};

use super::states::{ServiceStatesProbeNodeReplica, ServiceStatesProbeNodeReplicaLoad,
                    ServiceStatesProbeNodeReplicaReport};
use prober::manager::STORE as PROBER_STORE;
use prober::status::Status;
use prober::mode::Mode;

pub enum HandleError {
    InvalidLoad,
    WrongMode,
    NotFound,
}

pub fn handle(
    probe_id: &str,
    node_id: &str,
    replica_id: &str,
    interval: u64,
    load_cpu: f32,
    load_ram: f32,
) -> Result<(), HandleError> {
    debug!("report handle: {}:{}:{}", probe_id, node_id, replica_id);

    // Validate loads
    if load_cpu < 0.00 || load_ram < 0.00 {
        return Err(HandleError::InvalidLoad);
    }

    let mut store = PROBER_STORE.write().unwrap();

    if let Some(ref mut probe) = store.states.probes.get_mut(probe_id) {
        if let Some(ref mut node) = probe.nodes.get_mut(node_id) {
            // Mode isnt push? Dont accept report
            if node.mode != Mode::Push {
                return Err(HandleError::WrongMode);
            }

            // Acquire previous replica status (follow it up to avoid re-processing glitches)
            let status = if let Some(ref replica) = node.replicas.get(replica_id) {
                replica.status.to_owned()
            } else {
                Status::Healthy
            };

            // Bump stored replica
            node.replicas.insert(
                replica_id.to_string(),
                ServiceStatesProbeNodeReplica {
                    status: status,
                    url: None,
                    load: Some(ServiceStatesProbeNodeReplicaLoad {
                        cpu: load_cpu,
                        ram: load_ram,
                    }),
                    report: Some(ServiceStatesProbeNodeReplicaReport {
                        time: SystemTime::now(),
                        interval: Duration::from_secs(interval),
                    }),
                },
            );

            return Ok(());
        }
    }

    warn!(
        "report could not be stored: {}:{}:{}",
        probe_id,
        node_id,
        replica_id
    );

    Err(HandleError::NotFound)
}
