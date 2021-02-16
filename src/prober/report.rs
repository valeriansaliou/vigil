// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use log::{debug, warn};
use std::time::{Duration, SystemTime};

use super::states::{
    ServiceStatesProbeNodeReplica, ServiceStatesProbeNodeReplicaLoad,
    ServiceStatesProbeNodeReplicaLoadQueue, ServiceStatesProbeNodeReplicaMetrics,
    ServiceStatesProbeNodeReplicaMetricsSystem, ServiceStatesProbeNodeReplicaReport,
};
use crate::prober::manager::STORE as PROBER_STORE;
use crate::prober::mode::Mode;
use crate::prober::status::Status;

pub enum HandleLoadError {
    InvalidLoad,
    WrongMode,
    NotFound,
}

pub enum HandleHealthError {
    WrongMode,
    NotFound,
}

pub fn handle_load(
    probe_id: &str,
    node_id: &str,
    replica_id: &str,
    interval: u64,
    load_cpu: f32,
    load_ram: f32,
) -> Result<Option<String>, HandleLoadError> {
    debug!(
        "load report handle: {}:{}:{}",
        probe_id, node_id, replica_id
    );

    // Validate loads
    if load_cpu < 0.00 || load_ram < 0.00 {
        return Err(HandleLoadError::InvalidLoad);
    }

    let mut store = PROBER_STORE.write().unwrap();

    if let Some(ref mut probe) = store.states.probes.get_mut(probe_id) {
        if let Some(ref mut node) = probe.nodes.get_mut(node_id) {
            // Mode isnt push? Dont accept report
            if node.mode != Mode::Push {
                return Err(HandleLoadError::WrongMode);
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
                    script: None,
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
        "load report could not be stored: {}:{}:{}",
        probe_id, node_id, replica_id
    );

    Err(HandleLoadError::NotFound)
}

pub fn handle_health(
    probe_id: &str,
    node_id: &str,
    replica_id: &str,
    interval: u64,
    health: &Status,
) -> Result<(), HandleHealthError> {
    debug!(
        "health report handle: {}:{}:{}",
        probe_id, node_id, replica_id
    );

    let mut store = PROBER_STORE.write().unwrap();

    if let Some(ref mut probe) = store.states.probes.get_mut(probe_id) {
        if let Some(ref mut node) = probe.nodes.get_mut(node_id) {
            // Mode isnt local? Dont accept report
            if node.mode != Mode::Local {
                return Err(HandleHealthError::WrongMode);
            }

            // Bump stored replica
            node.replicas.insert(
                replica_id.to_string(),
                ServiceStatesProbeNodeReplica {
                    status: health.to_owned(),
                    url: None,
                    script: None,
                    metrics: ServiceStatesProbeNodeReplicaMetrics::default(),
                    load: None,
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
        "health report could not be stored: {}:{}:{}",
        probe_id, node_id, replica_id
    );

    Err(HandleHealthError::NotFound)
}
