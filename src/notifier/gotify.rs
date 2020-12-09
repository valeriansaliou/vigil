// Vigil
//
// Microservices Status Page
// Copyright: 2019, Valerian Saliou <valerian@valeriansaliou.name>
// Copyright: 2020, Rachel Chen <rachel@chens.email> - Modified based on pushover.rs
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;
use std::time::Duration;

use reqwest::blocking::Client;

use super::generic::{GenericNotifier, Notification, DISPATCH_TIMEOUT_SECONDS};
use crate::config::config::ConfigNotify;
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
                message.push_str("This is a startup alert.\n\n");
            } else if notification.changed == false {
                message.push_str("This is a reminder.\n\n");
            }

            message.push_str(&format!(
                "Status: {}\n",
                notification.status.as_str().to_uppercase()
            ));
            message.push_str(&format!(
                "Nodes:\n{}\n",
                &notification.replicas.join("\n")
            ));
            message.push_str(&format!("Time: {}", &notification.time));

            debug!("will send Gotify notification with message: {}", &message);

            // https://gotify.net/docs/pushmsg
            // TODO: render content as markdown
            let url = format!("{}/message?token={}", gotify.app_url.as_str(), gotify.app_token);
            let mut params: HashMap<&str, &str> = HashMap::new();
            params.insert("title", &APP_CONF.branding.page_title);
            params.insert("message", &message);

            if notification.changed == false {
                params.insert("priority", "10");
            }

            let response = GOTIFY_HTTP_CLIENT
                .post(&url)
                .form(&params)
                .send();

            if let Ok(response_inner) = response {
                if response_inner.status().is_success() != true {
                    return Err(true);
                }
            }else{
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
