// Vigil
//
// Microservices Status Page
// Copyright: 2019, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;
use std::time::Duration;

use reqwest::blocking::Client;

use super::generic::{GenericNotifier, Notification, DISPATCH_TIMEOUT_SECONDS};
use crate::config::config::ConfigNotify;
use crate::prober::status::Status;
use crate::APP_CONF;

lazy_static! {
    static ref GOTIFY_HTTP_CLIENT: Client = Client::builder()
        .timeout(Duration::from_secs(DISPATCH_TIMEOUT_SECONDS))
        .gzip(true)
        .build()
        .unwrap();
}

pub struct GotifyNotifier;

impl GenericNotifier for GotifyNotifier {
    fn attempt(notify: &ConfigNotify, notification: &Notification) -> Result<(), bool> {
        if let Some(ref gotify) = notify.gotify {
            // Build up the message text
            let mut message = String::new();

            if notification.startup == true {
                message.push_str("<b><i>This is a startup alert.</i></b>\n\n");
            } else if notification.changed == false {
                message.push_str("<b><i>This is a reminder.</i></b>\n\n");
            }

            message.push_str(&format!(
                "<u>Status:</u> <b><font color=\"{}\">{}</font></b>\n",
                status_to_color(&notification.status),
                notification.status.as_str().to_uppercase()
            ));
            message.push_str(&format!(
                "<u>Nodes:</u> {}\n",
                &notification.replicas.join(", ")
            ));
            message.push_str(&format!("<u>Time:</u> {}", &notification.time));

            debug!("will send Pushover notification with message: {}", &message);

            let mut has_sub_delivery_failure = false;

            if has_sub_delivery_failure == true {
                return Err(true);
            }

            return Ok(());
        }

        Err(false)
    }

    fn can_notify(notify: &ConfigNotify, notification: &Notification) -> bool {
        if let Some(ref gotify_config) = notify.gotify {
            notification.expected(gotify_config.reminders_only)
        } else {
            false
        }
    }

    fn name() -> &'static str {
        "gotify"
    }
}

fn status_to_color(status: &Status) -> &'static str {
    match status {
        &Status::Healthy => "#54A158",
        &Status::Sick => "#D5A048",
        &Status::Dead => "#C4291C",
    }
}
