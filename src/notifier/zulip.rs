// Vigil
//
// Microservices Status Page
// Copyright: 2021, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::Duration;

use reqwest::blocking::Client;
use serde_derive::Serialize;

use super::generic::{GenericNotifier, Notification, DISPATCH_TIMEOUT_SECONDS};
use crate::config::config::ConfigNotify;
use crate::prober::status::Status;
use crate::APP_CONF;

lazy_static::lazy_static! {
    static ref ZULIP_HTTP_CLIENT: Client = Client::builder()
        .timeout(Duration::from_secs(DISPATCH_TIMEOUT_SECONDS))
        .gzip(true)
        .build()
        .unwrap();
}

pub struct ZulipNotifier;

#[derive(Serialize)]
struct ZulipPayload<'a> {
    #[serde(rename(serialize = "type"))]
    type_: &'a str,
    to: &'a str,
    topic: &'a str,
    content: &'a str,
}

impl GenericNotifier for ZulipNotifier {
    fn attempt(notify: &ConfigNotify, notification: &Notification) -> Result<(), bool> {
        if let Some(ref zulip) = notify.zulip {
            let status_label = format!("{:?}", notification.status);

            let status_str = match notification.status {
                Status::Dead => " *dead* :boom:",
                Status::Healthy => " *healthy* :check_mark:",
                Status::Sick => " *sick* :sick:",
            };

            // Build message
            let mut message_text = if notification.startup == true {
                format!("Status started up, as: {}.", status_str)
            } else if notification.changed {
                format!("Status changed to: {}.", status_str)
            } else {
                format!("Status is still: {}.", status_str)
            };

            if notification.replicas.len() > 0 {
                let nodes_label = notification.replicas.join(", ");
                let nodes_label_titled = format!("\n **Nodes**: *{}*.", nodes_label);

                message_text.push_str(&nodes_label_titled);
            }

            message_text.push_str(&format!("\n **Status**: {}", &status_label));
            message_text.push_str(&format!("\n **Time**: {}", &notification.time));
            message_text.push_str(&format!(
                "\n **Page**: {}",
                &APP_CONF.branding.page_url.as_str()
            ));

            // Submit payload to Zulip
            let payload = ZulipPayload {
                type_: "stream",
                to: &zulip.channel,
                topic: "Vigil status",
                content: &message_text,
            };

            let response = ZULIP_HTTP_CLIENT
                .post(zulip.api_url.join("messages").unwrap().as_str())
                .basic_auth(zulip.bot_email.clone(), Some(zulip.bot_api_key.clone()))
                .form(&payload)
                .send();

            if let Ok(response_inner) = response {
                if response_inner.status().is_success() == true {
                    return Ok(());
                } else {
                    log::warn!(
                        "Error while submitting data to zulip: {:?}",
                        response_inner.text()
                    );
                }
            }

            return Err(true);
        }

        Err(false)
    }

    fn can_notify(notify: &ConfigNotify, notification: &Notification) -> bool {
        if let Some(ref zulip_config) = notify.zulip {
            notification.expected(zulip_config.reminders_only)
        } else {
            false
        }
    }

    fn name() -> &'static str {
        "zulip"
    }
}
