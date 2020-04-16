use std::collections::HashMap;
use std::time::Duration;

use reqwest::blocking::Client;

use super::generic::{GenericNotifier, Notification, DISPATCH_TIMEOUT_SECONDS};
use crate::config::config::ConfigNotify;
use crate::prober::status::Status;
use crate::APP_CONF;

lazy_static! {
    static ref TELEGRAM_HTTP_CLIENT: Client = Client::builder()
        .timeout(Duration::from_secs(DISPATCH_TIMEOUT_SECONDS))
        .gzip(true)
        .build()
        .unwrap();
}

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
            let status_icon = match &notification.status {
                Status::Dead => "\u{274c}",
                Status::Sick => "\u{26a0}",
                Status::Healthy => "\u{2705}",
            };

            let mut message_text = if notification.changed == true {
                format!(
                    "{} Status changed to *{}*.\n",
                    status_icon,
                    notification.status.as_str()
                )
            } else {
                format!(
                    "{} Status is still *{}*.\n",
                    status_icon,
                    notification.status.as_str()
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

            message_text.push_str(&nodes_count_list_text);
            message_text.push_str(&format!("\nLink: {}", APP_CONF.branding.page_url.as_str()));

            let chat_id = match &telegram.chat_id.parse::<u64>() {
                Ok(user_chat_id) => TelegramChatID::User(*user_chat_id),
                Err(_) => TelegramChatID::Group(&telegram.chat_id.as_str()),
            };

            // Build payload
            let payload = TelegramPayload {
                chat_id: chat_id,
                text: message_text,
                parse_mode: "markdown",
                disable_web_page_preview: true,
            };

            let url = format!(
                "https://api.telegram.org/bot{}/sendMessage",
                telegram.bot_token
            );
            let response = TELEGRAM_HTTP_CLIENT
                .post(url.as_str())
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
