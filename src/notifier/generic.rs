// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prober::status::Status;
use config::config::ConfigNotify;

pub const DISPATCH_TIMEOUT_SECONDS: u64 = 10;

pub struct Notification<'a> {
    pub status: &'a Status,
    pub time: String,
    pub replicas: Vec<&'a str>,
}

pub trait GenericNotifier {
    fn dispatch(notify: &ConfigNotify, notification: &Notification) -> Result<(), bool>;
    fn is_enabled(notify: &ConfigNotify) -> bool;
}
