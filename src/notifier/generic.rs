// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::thread;
use std::time::Duration;

use crate::config::config::ConfigNotify;
use crate::prober::status::Status;

const DISPATCH_TRY_WAIT_SECONDS: u64 = 2;
const DISPATCH_TRY_ATTEMPT_TIMES: u8 = 3;
pub const DISPATCH_TIMEOUT_SECONDS: u64 = 10;

pub struct Notification<'a> {
    pub status: &'a Status,
    pub time: String,
    pub replicas: Vec<&'a str>,
    pub changed: bool,
}

pub trait GenericNotifier {
    fn attempt(notify: &ConfigNotify, notification: &Notification) -> Result<(), bool>;
    fn can_notify(notify: &ConfigNotify, notification: &Notification) -> bool;
    fn name() -> &'static str;
}

impl<'a> Notification<'a> {
    pub fn dispatch<N: GenericNotifier>(
        notify: &ConfigNotify,
        notification: &Notification,
    ) -> Result<(), bool> {
        if N::can_notify(notify, notification) == true {
            info!(
                "dispatch {} notification for status: {:?} and replicas: {:?}",
                N::name(),
                notification.status,
                notification.replicas
            );

            for try_index in 1..(DISPATCH_TRY_ATTEMPT_TIMES + 1) {
                debug!(
                    "dispatch {} notification attempt: #{}",
                    N::name(),
                    try_index
                );

                // Hold on for next try
                if try_index > 1 {
                    thread::sleep(Duration::from_secs(DISPATCH_TRY_WAIT_SECONDS))
                }

                // Attempt notification dispatch
                if N::attempt(notify, notification).is_ok() == true {
                    debug!("dispatched notification to provider: {}", N::name());

                    return Ok(());
                }
            }

            error!("failed dispatching notification to provider: {}", N::name());

            return Err(true);
        }

        debug!("did not dispatch notification to provider: {}", N::name());

        Err(false)
    }

    pub fn expected(&self, reminders_only: bool) -> bool {
        // Notification may not be expected if status has changed, but we only want to receive \
        //   reminders on this specific notifier channel.
        if reminders_only == false || (reminders_only == true && self.changed == false) {
            true
        } else {
            false
        }
    }
}
