// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::thread;
use std::time::{SystemTime, Duration};
use std::iter::FromIterator;
use time;

use prober::status::Status;
use prober::mode::Mode;
use prober::manager::{STORE as PROBER_STORE};
use notifier::generic::{Notification, GenericNotifier};
use notifier::email::EmailNotifier;
use notifier::slack::SlackNotifier;
use APP_CONF;

const AGGREGATE_INTERVAL_SECONDS: u64 = 10;

struct BumpedStates {
    status: Status,
    replicas: Vec<String>,
}

fn check_child_status(parent_status: &Status, child_status: &Status) -> Option<Status> {
    if child_status == &Status::Dead {
        Some(Status::Dead)
    } else if child_status == &Status::Sick && parent_status != &Status::Dead {
        Some(Status::Sick)
    } else {
        None
    }
}

fn scan_and_bump_states() -> Option<BumpedStates> {
    let mut bumped_replicas = Vec::new();

    let mut store = PROBER_STORE.write().unwrap();

    let mut general_status = Status::Healthy;

    for (probe_id, probe) in store.states.probes.iter_mut() {
        debug!("aggregate probe: {}", probe_id);

        let mut probe_status = Status::Healthy;

        for (node_id, node) in probe.nodes.iter_mut() {
            debug!("aggregate node: {}:{}", probe_id, node_id);

            let mut node_status = Status::Healthy;

            for (replica_id, replica) in node.replicas.iter_mut() {
                let mut replica_status = Status::Healthy;

                // Process push metrics?
                if node.mode == Mode::Push {
                    // Compare delays and compute a new status?
                    if let Some(ref replica_report) = replica.report {
                        if let Ok(duration_since_report) = SystemTime::now().duration_since(
                            replica_report.time) {
                            if duration_since_report >= (replica_report.interval +
                                Duration::from_secs(APP_CONF.metrics.push_delay_dead)) {
                                debug!(
                                    "replica: {}:{}:{} is dead because it didnt report in a while",
                                    probe_id, node_id, replica_id
                                );

                                replica_status = Status::Dead;
                            }
                        }
                    }

                    // Compare system load indices and compute a new status?
                    if replica_status == Status::Healthy {
                        if let Some(ref replica_load) = replica.load {
                            if (replica_load.cpu > APP_CONF.metrics.push_system_cpu_sick_above) ||
                                (replica_load.ram > APP_CONF.metrics.push_system_ram_sick_above) {
                                debug!(
                                    "replica: {}:{}:{} is sick because it is overloaded",
                                    probe_id, node_id, replica_id
                                );

                                replica_status = Status::Sick;
                            }
                        }
                    }
                } else {
                    replica_status = replica.status.to_owned();
                }

                // Bump node status with worst replica status?
                if let Some(worst_status) = check_child_status(&node_status, &replica_status) {
                    node_status = worst_status;
                }

                debug!(
                    "aggregated status for replica: {}:{}:{} => {:?}",
                    probe_id, node_id, replica_id, replica_status
                );

                // Append bumped replica path?
                if replica.status != replica_status {
                    bumped_replicas.push(format!("{}:{}:{}", probe_id, node_id, replica_id));
                }

                replica.status = replica_status;
            }

            // Bump probe status with worst node status?
            if let Some(worst_status) = check_child_status(&probe_status, &node_status) {
                probe_status = worst_status;
            }

            debug!("aggregated status for node: {}:{} => {:?}", probe_id, node_id, node_status);

            node.status = node_status;
        }

        // Bump general status with worst node status?
        if let Some(worst_status) = check_child_status(&general_status, &probe_status) {
            general_status = worst_status;
        }

        debug!("aggregated status for probe: {} => {:?}", probe_id, probe_status);

        probe.status = probe_status;
    }

    // Check if should dispatch notification later (only if critical)
    // Allow for cases:
    //   - healthy >> dead
    //   - sick    >> dead
    //   - dead    >> sick
    //   - dead    >> healthy
    let should_notify = (store.states.status != Status::Dead && general_status == Status::Dead) ||
        (store.states.status == Status::Dead && general_status != Status::Dead);

    // Bump stored values
    store.states.status = general_status.to_owned();

    if let Ok(time_string) = time_now_as_string() {
        store.states.date = Some(time_string);
    }

    if should_notify == true && bumped_replicas.len() > 0 {
        Some(BumpedStates {
            status: general_status,
            replicas: bumped_replicas,
        })
    } else {
        None
    }
}

fn time_now_as_string() -> Result<String, ()> {
    time::strftime("%H:%M:%S UTC%z", &time::now()).or(Err(()))
}

pub fn run() {
    loop {
        debug!("running an aggregate operation...");

        // Should notify after bump?
        let bumped_states = scan_and_bump_states();

        if let Some(ref bumped_states_inner) = bumped_states {
            let notification = Notification {
                status: &bumped_states_inner.status,
                time: time_now_as_string().unwrap_or("".to_string()),
                replicas: Vec::from_iter(bumped_states_inner.replicas.iter().map(String::as_str)),
            };

            for result in [
                ("email", EmailNotifier::dispatch(&notification)),
                ("slack", SlackNotifier::dispatch(&notification))
            ].iter() {
                if result.1.is_ok() == true {
                    debug!("dispatched notification to provider: {}", result.0);
                } else {
                    if let Err(true) = result.1 {
                        error!("failed dispatching notification to provider: {}", result.0);
                    } else {
                        debug!("did not dispatch notification to provider: {}", result.0);
                    }
                }
            }
        }

        info!("ran aggregate operation (notified: {})", bumped_states.is_some());

        // Hold for next aggregate run
        thread::sleep(Duration::from_secs(AGGREGATE_INTERVAL_SECONDS));
    }
}
