// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::{Duration, SystemTime};

use super::states::{
    ServiceStatesProbeNodeReplica, ServiceStatesProbeNodeReplicaLoad,
    ServiceStatesProbeNodeReplicaLoadQueue, ServiceStatesProbeNodeReplicaMetrics,
    ServiceStatesProbeNodeReplicaMetricsSystem, ServiceStatesProbeNodeReplicaReport,
};
use crate::prober::manager::STORE as PROBER_STORE;
use crate::prober::mode::Mode;
use crate::prober::status::Status;

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
) -> Result<Option<String>, HandleError> {
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

            // Acquire previous replica status + previous queue load status (follow-up values)
            let (status, mut metrics, mut load_queue);

            load_queue = ServiceStatesProbeNodeReplicaLoadQueue::default();

            if let Some(ref replica) = node.replicas.get(replica_id) {
                status = replica.status.to_owned();
                metrics = replica.metrics.to_owned();

                if let Some(ref replica_load) = replica.load {
                    load_queue = replica_load.queue.clone();
                }
            } else {
                status = Status::Healthy;
                metrics = ServiceStatesProbeNodeReplicaMetrics::default();
            }

            // Assign new system metrics
            metrics.system = Some(ServiceStatesProbeNodeReplicaMetricsSystem {
                cpu: (load_cpu * 100.0).round() as u16,
                ram: (load_ram * 100.0).round() as u16,
            });

            // Bump stored replica
            node.replicas.insert(
                replica_id.to_string(),
                ServiceStatesProbeNodeReplica {
                    status: status,
                    url: None,
                    metrics: metrics,
                    load: Some(ServiceStatesProbeNodeReplicaLoad {
                        cpu: load_cpu,
                        ram: load_ram,
                        queue: load_queue,
                    }),
                    report: Some(ServiceStatesProbeNodeReplicaReport {
                        time: SystemTime::now(),
                        interval: Duration::from_secs(interval),
                    }),
                },
            );

            return Ok(node.rabbitmq_queue.to_owned());
        }
    }

    warn!(
        "report could not be stored: {}:{}:{}",
        probe_id, node_id, replica_id
    );

    Err(HandleError::NotFound)
}
