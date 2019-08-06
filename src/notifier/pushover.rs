// Vigil
//
// Microservices Status Page
// Copyright: 2019, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;
use std::time::Duration;

use reqwest::Client;

use super::generic::{GenericNotifier, Notification, DISPATCH_TIMEOUT_SECONDS};
use crate::config::config::ConfigNotify;
use crate::prober::status::Status;
use crate::APP_CONF;

lazy_static! {
    static ref PUSHOVER_HTTP_CLIENT: Client = Client::builder()
        .timeout(Duration::from_secs(DISPATCH_TIMEOUT_SECONDS))
        .gzip(true)
        .build()
        .unwrap();
}

static PUSHOVER_API_URL: &'static str = "https://api.pushover.net/1/messages.json";

pub struct PushoverNotifier;

impl GenericNotifier for PushoverNotifier {
    fn attempt(notify: &ConfigNotify, notification: &Notification) -> Result<(), bool> {
        if let Some(ref pushover) = notify.pushover {
            // Build up the message text
            let mut message = String::new();

            if notification.changed == false {
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

            for user_key in &pushover.user_keys {
                // Build form parameters
                let mut params: HashMap<&str, &str> = HashMap::new();

                // Append authorization values
                params.insert("token", &pushover.app_token);
                params.insert("user", user_key);

                // Append title & message
                params.insert("title", &APP_CONF.branding.page_title);
                params.insert("message", &message);
                params.insert("html", "1");

                // Append target URL
                let url_title = format!("Details on {}", APP_CONF.branding.page_title);

                params.insert("url_title", &url_title);
                params.insert("url", APP_CONF.branding.page_url.as_str());

                // Mark as high-priority? (reminder)
                if notification.changed == false {
                    params.insert("priority", "1");
                }

                // Submit message to Pushover
                let response = PUSHOVER_HTTP_CLIENT
                    .post(PUSHOVER_API_URL)
                    .form(&params)
                    .send();

                // Check for any failure
                if let Ok(response_inner) = response {
                    if response_inner.status().is_success() != true {
                        has_sub_delivery_failure = true;
                    }
                } else {
                    has_sub_delivery_failure = true;
                }
            }

            if has_sub_delivery_failure == true {
                return Err(true);
            }

            return Ok(());
        }

        Err(false)
    }

    fn can_notify(notify: &ConfigNotify, notification: &Notification) -> bool {
        if let Some(ref pushover_config) = notify.pushover {
            notification.expected(pushover_config.reminders_only)
        } else {
            false
        }
    }

    fn name() -> &'static str {
        "pushover"
    }
}

fn status_to_color(status: &Status) -> &'static str {
    match status {
        &Status::Healthy => "#54A158",
        &Status::Sick => "#D5A048",
        &Status::Dead => "#C4291C",
    }
}
