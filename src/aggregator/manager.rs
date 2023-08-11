// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::iter::FromIterator;
use std::thread;
use std::time::{Duration, SystemTime};
use time;
use time::format_description::FormatItem;

use crate::config::config::ConfigNotifyReminderBackoffFunction;
use crate::notifier::generic::Notification;
use crate::prober::manager::STORE as PROBER_STORE;
use crate::prober::mode::Mode;
use crate::prober::status::Status;
use crate::APP_CONF;

#[cfg(feature = "notifier-email")]
use crate::notifier::email::EmailNotifier;

#[cfg(feature = "notifier-twilio")]
use crate::notifier::twilio::TwilioNotifier;

#[cfg(feature = "notifier-slack")]
use crate::notifier::slack::SlackNotifier;

#[cfg(feature = "notifier-zulip")]
use crate::notifier::zulip::ZulipNotifier;

#[cfg(feature = "notifier-telegram")]
use crate::notifier::telegram::TelegramNotifier;

#[cfg(feature = "notifier-pushover")]
use crate::notifier::pushover::PushoverNotifier;

#[cfg(feature = "notifier-gotify")]
use crate::notifier::gotify::GotifyNotifier;

#[cfg(feature = "notifier-xmpp")]
use crate::notifier::xmpp::XMPPNotifier;

#[cfg(feature = "notifier-matrix")]
use crate::notifier::matrix::MatrixNotifier;

#[cfg(feature = "notifier-webex")]
use crate::notifier::webex::WebExNotifier;

#[cfg(feature = "notifier-webhook")]
use crate::notifier::webhook::WebHookNotifier;

lazy_static! {
    static ref TIME_NOW_FORMATTER: Vec<FormatItem<'static>> = time::format_description::parse(
        "[hour]:[minute]:[second] UTC[offset_hour sign:mandatory]:[offset_minute]"
    )
    .expect("invalid time format");
}

const AGGREGATE_INTERVAL_SECONDS: u64 = 10;

struct BumpedStates {
    status: Status,
    replicas: Vec<String>,
    changed: bool,
    startup: bool,
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

                // Process metrics
                match node.mode {
                    Mode::Push => {
                        // Compare delays and compute a new status?
                        if let Some(ref replica_report) = replica.report {
                            if let Ok(duration_since_report) =
                                SystemTime::now().duration_since(replica_report.time)
                            {
                                if duration_since_report
                                    >= (replica_report.interval
                                        + Duration::from_secs(APP_CONF.metrics.push_delay_dead))
                                {
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
                                if (replica_load.cpu > APP_CONF.metrics.push_system_cpu_sick_above)
                                    || (replica_load.ram
                                        > APP_CONF.metrics.push_system_ram_sick_above)
                                {
                                    debug!(
                                        "replica: {}:{}:{} is sick because it is overloaded",
                                        probe_id, node_id, replica_id
                                    );

                                    replica_status = Status::Sick;
                                }
                            }
                        }

                        // Check RabbitMQ queue full marker?
                        if replica_status == Status::Healthy {
                            if let Some(ref replica_load) = replica.load {
                                if replica_load.queue.stalled == true {
                                    replica_status = Status::Dead;
                                } else if replica_load.queue.loaded == true {
                                    replica_status = Status::Sick;
                                }
                            }
                        }
                    }
                    Mode::Local => {
                        // Assign stored status by default ('local' nodes report their status \
                        //   themselves)
                        replica_status = replica.status.to_owned();

                        // Compare delays and compute a new status?
                        if let Some(ref replica_report) = replica.report {
                            if let Ok(duration_since_report) =
                                SystemTime::now().duration_since(replica_report.time)
                            {
                                if duration_since_report
                                    >= (replica_report.interval
                                        + Duration::from_secs(APP_CONF.metrics.local_delay_dead))
                                {
                                    debug!(
                                        "replica: {}:{}:{} is dead because it didnt report in a while",
                                        probe_id, node_id, replica_id
                                    );

                                    replica_status = Status::Dead;
                                }
                            }
                        }
                    }
                    _ => {
                        // Forward stored status (eg. 'poll' or 'script' nodes)
                        replica_status = replica.status.to_owned();
                    }
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
                if replica_status == Status::Dead {
                    bumped_replicas.push(format!("{}:{}:{}", probe_id, node_id, replica_id));
                }

                replica.status = replica_status;
            }

            // Bump probe status with worst node status?
            if let Some(worst_status) = check_child_status(&probe_status, &node_status) {
                probe_status = worst_status;
            }

            debug!(
                "aggregated status for node: {}:{} => {:?}",
                probe_id, node_id, node_status
            );

            node.status = node_status;
        }

        // Bump general status with worst node status?
        if let Some(worst_status) = check_child_status(&general_status, &probe_status) {
            general_status = worst_status;
        }

        debug!(
            "aggregated status for probe: {} => {:?}",
            probe_id, probe_status
        );

        probe.status = probe_status;
    }

    // Check if general status has changed
    let has_changed = store.states.status != general_status;

    // Check if should dispatch notification later (only if critical)
    // Allow for cases:
    //   - healthy >> dead
    //   - sick    >> dead
    //   - dead    >> sick
    //   - dead    >> healthy
    let mut should_notify = (store.states.status != Status::Dead && general_status == Status::Dead)
        || (store.states.status == Status::Dead && general_status != Status::Dead);

    // Reset the backoff counter whenever we are not dead (yet, stored status changed)
    if has_changed == true && general_status != Status::Dead {
        store.states.notifier.reminder_backoff_counter = 1;
    }

    // Check if should re-notify? (in case status did not change; only if dead)
    // Notice: this is used to send periodic reminders of downtime (ie. 'still down' messages)
    if has_changed == false && should_notify == false && general_status == Status::Dead {
        debug!("status unchanged, but may need to re-notify; checking");

        if let Some(ref notify) = APP_CONF.notify {
            match (store.notified, notify.reminder_interval) {
                (Some(last_notified), Some(reminder_interval)) => {
                    if let Ok(duration_since_notified) =
                        SystemTime::now().duration_since(last_notified)
                    {
                        // Notice: we use backoff counter all the time because if it is disabled, \
                        //   then the value is 1 at any time, thus not impacting the interval.
                        let reminder_backoff_counter =
                            store.states.notifier.reminder_backoff_counter;
                        let reminder_ignore_until = store.states.notifier.reminder_ignore_until;
                        let reminder_interval_backoff = Duration::from_secs(
                            reminder_interval
                                * (reminder_backoff_counter as u64)
                                    .pow(notify.reminder_backoff_function as u32),
                        );

                        // Check if reminders should be ignored for now?
                        let should_ignore_reminders =
                            if let Some(reminder_ignore_until) = reminder_ignore_until {
                                SystemTime::now() < reminder_ignore_until
                            } else {
                                false
                            };

                        debug!(
                            "checking if should re-notify about unchanged status ({}s / {}↑ / {})",
                            reminder_interval_backoff.as_secs(),
                            reminder_backoff_counter,
                            if should_ignore_reminders == false {
                                "✓"
                            } else {
                                "✖"
                            }
                        );

                        // Duration since last notified exceeds reminder interval? Should re-notify
                        if duration_since_notified >= reminder_interval_backoff
                            && should_ignore_reminders == false
                        {
                            info!("should re-notify about unchanged status");

                            should_notify = true;

                            // Increment the backoff counter? (a backoff function is set, \
                            //   therefore reminders backoff is enabled)
                            if notify.reminder_backoff_function
                                != ConfigNotifyReminderBackoffFunction::None
                                && store.states.notifier.reminder_backoff_counter
                                    < notify.reminder_backoff_limit
                            {
                                store.states.notifier.reminder_backoff_counter += 1;

                                debug!(
                                    "incremented re-notify backoff counter to: {} (limit: {})",
                                    store.states.notifier.reminder_backoff_counter,
                                    notify.reminder_backoff_limit
                                );
                            }
                        } else {
                            debug!(
                                "should not re-notify about unchanged status (interval: {})",
                                reminder_interval
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Bump stored values
    store.states.status = general_status.to_owned();
    store.states.date = Some(time_now_as_string());

    if should_notify == true {
        store.notified = Some(SystemTime::now());

        Some(BumpedStates {
            status: general_status,
            replicas: bumped_replicas,
            changed: has_changed,
            startup: false,
        })
    } else {
        None
    }
}

fn time_now_as_string() -> String {
    time::OffsetDateTime::now_utc()
        .format(&TIME_NOW_FORMATTER)
        .unwrap_or("?".to_string())
}

fn dispatch_startup_notification() {
    if let Some(ref conf_notify) = APP_CONF.notify {
        if conf_notify.startup_notification == true {
            debug!("sending aggregate startup notification...");

            notify(&BumpedStates {
                status: Status::Healthy,
                replicas: Vec::new(),
                changed: true,
                startup: true,
            });
        }
    }
}

fn notify(bumped_states: &BumpedStates) {
    let notification = Notification {
        status: &bumped_states.status,
        time: time_now_as_string(),
        replicas: Vec::from_iter(bumped_states.replicas.iter().map(String::as_str)),
        changed: bumped_states.changed,
        startup: bumped_states.startup,
    };

    if let Some(ref notify) = APP_CONF.notify {
        #[cfg(feature = "notifier-email")]
        Notification::dispatch::<EmailNotifier>(notify, &notification).ok();

        #[cfg(feature = "notifier-twilio")]
        Notification::dispatch::<TwilioNotifier>(notify, &notification).ok();

        #[cfg(feature = "notifier-slack")]
        Notification::dispatch::<SlackNotifier>(notify, &notification).ok();

        #[cfg(feature = "notifier-zulip")]
        Notification::dispatch::<ZulipNotifier>(notify, &notification).ok();

        #[cfg(feature = "notifier-telegram")]
        Notification::dispatch::<TelegramNotifier>(notify, &notification).ok();

        #[cfg(feature = "notifier-pushover")]
        Notification::dispatch::<PushoverNotifier>(notify, &notification).ok();

        #[cfg(feature = "notifier-gotify")]
        Notification::dispatch::<GotifyNotifier>(notify, &notification).ok();

        #[cfg(feature = "notifier-xmpp")]
        Notification::dispatch::<XMPPNotifier>(notify, &notification).ok();

        #[cfg(feature = "notifier-matrix")]
        Notification::dispatch::<MatrixNotifier>(notify, &notification).ok();

        #[cfg(feature = "notifier-webex")]
        Notification::dispatch::<WebExNotifier>(notify, &notification).ok();

        #[cfg(feature = "notifier-webhook")]
        Notification::dispatch::<WebHookNotifier>(notify, &notification).ok();
    }
}

pub fn run() {
    // Notify that systems are healthy (when booting up aggregator)
    dispatch_startup_notification();

    // Start aggregate loop
    loop {
        debug!("running an aggregate operation...");

        // Should notify after bump?
        let bumped_states = scan_and_bump_states();

        if let Some(ref bumped_states_inner) = bumped_states {
            notify(bumped_states_inner);
        }

        info!(
            "ran aggregate operation (notified: {})",
            bumped_states.is_some()
        );

        // Hold for next aggregate run
        thread::sleep(Duration::from_secs(AGGREGATE_INTERVAL_SECONDS));
    }
}
