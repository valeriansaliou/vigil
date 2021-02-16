// Vigil
//
// Microservices Status Page
// Copyright: 2019, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use log::debug;
use std::collections::HashMap;
use std::time::Duration;

use serde_derive::Serialize;

use reqwest::blocking::Client;

use super::generic::{GenericNotifier, Notification, DISPATCH_TIMEOUT_SECONDS};
use crate::config::config::ConfigNotify;
use crate::APP_CONF;

lazy_static::lazy_static! {
    static ref TELEGRAM_HTTP_CLIENT: Client = Client::builder()
        .timeout(Duration::from_secs(DISPATCH_TIMEOUT_SECONDS))
        .gzip(true)
        .build()
        .unwrap();
}

static TELEGRAM_API_BASE_URL: &'static str = "https://api.telegram.org";

pub struct TelegramNotifier;

#[derive(Serialize)]
struct TelegramPayload<'a> {
    chat_id: TelegramChatID<'a>,
    text: String,
    parse_mode: &'static str,
    disable_web_page_preview: bool,
}

#[derive(Serialize)]
#[serde(untagged)]
enum TelegramChatID<'a> {
    Group(&'a str),
    User(u64),
}

impl GenericNotifier for TelegramNotifier {
    fn attempt(notify: &ConfigNotify, notification: &Notification) -> Result<(), bool> {
        if let Some(ref telegram) = notify.telegram {
            // Build message
            let mut message = if notification.startup == true {
                format!(
                    "{} Status started up, as: *{}*.\n",
                    notification.status.as_icon(),
                    notification.status.as_str().to_uppercase()
                )
            } else if notification.changed == true {
                format!(
                    "{} Status changed to: *{}*.\n",
                    notification.status.as_icon(),
                    notification.status.as_str().to_uppercase()
                )
            } else {
                format!(
                    "{} Status is still: *{}*.\n",
                    notification.status.as_icon(),
                    notification.status.as_str().to_uppercase()
                )
            };

            let mut replicas_count: HashMap<String, u32> = HashMap::new();

            for replica in notification.replicas.iter() {
                let service_and_node = replica.split(":").take(2).collect::<Vec<&str>>().join(":");
                *replicas_count.entry(service_and_node).or_insert(0) += 1;
            }

            let nodes_count_list_text = replicas_count
                .iter()
                .map(|(service_and_node, count)| {
                    format!(
                        "- `{}`: {} {}",
                        service_and_node,
                        count,
                        notification.status.as_str()
                    )
                })
                .collect::<Vec<String>>()
                .join("\n");

            message.push_str(&nodes_count_list_text);
            message.push_str(&format!("\nLink: {}", APP_CONF.branding.page_url.as_str()));

            debug!("will send Telegram notification with message: {}", &message);

            // Generate Telegram chat identifier
            let chat_id = match &telegram.chat_id.parse::<u64>() {
                Ok(user_chat_id) => TelegramChatID::User(*user_chat_id),
                Err(_) => TelegramChatID::Group(&telegram.chat_id.as_str()),
            };

            // Build payload
            let payload = TelegramPayload {
                chat_id: chat_id,
                text: message,
                parse_mode: "markdown",
                disable_web_page_preview: true,
            };

            // Generate target API URL
            let url = format!(
                "{}/bot{}/sendMessage",
                TELEGRAM_API_BASE_URL, telegram.bot_token
            );

            // Submit message to Telegram
            let response = TELEGRAM_HTTP_CLIENT
                .post(url.as_str())
                .json(&payload)
                .send();

            // Check for any failure
            if let Ok(response_inner) = response {
                if response_inner.status().is_success() == true {
                    return Ok(());
                }
            }

            return Err(true);
        }

        Err(false)
    }

    fn can_notify(notify: &ConfigNotify, notification: &Notification) -> bool {
        if let Some(ref telegram_config) = notify.telegram {
            notification.expected(telegram_config.reminders_only)
        } else {
            false
        }
    }

    fn name() -> &'static str {
        "telegram"
    }
}
