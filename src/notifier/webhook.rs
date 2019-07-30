// Vigil
//
// Microservices Status Page
// Copyright: 2019, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::Duration;

use reqwest::Client;

use super::generic::{GenericNotifier, Notification, DISPATCH_TIMEOUT_SECONDS};
use crate::config::config::ConfigNotify;
use crate::prober::status::Status;
use crate::APP_CONF;

lazy_static! {
    static ref WEBHOOK_HTTP_CLIENT: Client = Client::builder()
        .timeout(Duration::from_secs(DISPATCH_TIMEOUT_SECONDS))
        .gzip(true)
        .build()
        .unwrap();
}

pub struct WebHookNotifier;

#[derive(Serialize)]
struct WebHookPayload<'a> {
    #[serde(rename = "type")]
    _type: WebHookPayloadType,

    status: &'a Status,
    time: &'a str,
    replicas: &'a [&'a str],
    page: WebHookPayloadPage<'a>,
}

#[derive(Serialize)]
pub enum WebHookPayloadType {
    #[serde(rename = "changed")]
    Changed,

    #[serde(rename = "reminder")]
    Reminder,
}

#[derive(Serialize)]
struct WebHookPayloadPage<'a> {
    title: &'a str,
    url: &'a str,
}

impl GenericNotifier for WebHookNotifier {
    fn attempt(notify: &ConfigNotify, notification: &Notification) -> Result<(), bool> {
        if let Some(ref webhook) = notify.webhook {
            // Acquire hook type
            let hook_type = if notification.changed == true {
                WebHookPayloadType::Changed
            } else {
                WebHookPayloadType::Reminder
            };

            // Build paylaod
            let payload = WebHookPayload {
                _type: hook_type,
                status: notification.status,
                time: notification.time.as_str(),
                replicas: &notification.replicas,
                page: WebHookPayloadPage {
                    title: APP_CONF.branding.page_title.as_str(),
                    url: APP_CONF.branding.page_url.as_str(),
                },
            };

            // Submit payload to Web Hooks
            let response = WEBHOOK_HTTP_CLIENT
                .post(webhook.hook_url.as_str())
                .json(&payload)
                .send();

            if let Ok(response_inner) = response {
                if response_inner.status().is_success() == true {
                    return Ok(());
                }
            }

            return Err(true);
        }

        Err(false)
    }

    fn can_notify(notify: &ConfigNotify, _: &Notification) -> bool {
        notify.webhook.is_some()
    }

    fn name() -> &'static str {
        "webhook"
    }
}
