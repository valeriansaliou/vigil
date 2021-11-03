// Vigil
//
// Microservices Status Page
// Copyright: 2021, Valerian Saliou <valerian@valeriansaliou.name>
// Copyright: 2021, Enrico Risa https://github.com/wolf4ood
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;
use std::time::Duration;

use reqwest::blocking::Client;

use super::generic::{GenericNotifier, Notification, DISPATCH_TIMEOUT_SECONDS};
use crate::config::config::ConfigNotify;
use crate::APP_CONF;

lazy_static! {
    static ref MATRIX_HTTP_CLIENT: Client = Client::builder()
        .timeout(Duration::from_secs(DISPATCH_TIMEOUT_SECONDS))
        .gzip(true)
        .build()
        .unwrap();
    static ref MATRIX_FORMATTERS: Vec<fn(&Notification) -> String> = vec![
        format_status,
        format_replicas,
        format_status_page,
        format_time
    ];
}

static MATRIX_MESSAGE_BODY: &'static str = "You received a Vigil alert.";
static MATRIX_MESSAGE_TYPE: &'static str = "m.text";
static MATRIX_MESSAGE_FORMAT: &'static str = "org.matrix.custom.html";

pub struct MatrixNotifier;

impl GenericNotifier for MatrixNotifier {
    fn attempt(notify: &ConfigNotify, notification: &Notification) -> Result<(), bool> {
        if let Some(ref matrix) = notify.matrix {
            // Build up the message text
            let message = format_message(notification);

            debug!("will send Matrix notification with message: {}", &message);

            // Generate URL
            // See: https://matrix.org/docs/guides/client-server-api#sending-messages
            let url = format!(
                "{}_matrix/client/r0/rooms/{}/send/m.room.message?access_token={}",
                matrix.homeserver_url.as_str(),
                matrix.room_id.as_str(),
                matrix.access_token.as_str()
            );

            // Build message parameters
            let mut params: HashMap<&str, &str> = HashMap::new();

            params.insert("body", MATRIX_MESSAGE_BODY);
            params.insert("msgtype", MATRIX_MESSAGE_TYPE);
            params.insert("format", MATRIX_MESSAGE_FORMAT);
            params.insert("formatted_body", &message);

            // Submit message to Matrix
            let response = MATRIX_HTTP_CLIENT.post(&url).json(&params).send();

            if let Ok(response_inner) = response {
                if response_inner.status().is_success() != true {
                    return Err(true);
                }
            } else {
                return Err(true);
            }

            return Ok(());
        }

        Err(false)
    }

    fn can_notify(notify: &ConfigNotify, notification: &Notification) -> bool {
        if let Some(ref matrix_config) = notify.matrix {
            notification.expected(matrix_config.reminders_only)
        } else {
            false
        }
    }

    fn name() -> &'static str {
        "matrix"
    }
}

fn format_status(notification: &Notification) -> String {
    let msg = if notification.startup == true {
        "Status started up, as"
    } else if notification.changed == true {
        "Status changed to"
    } else {
        "Status is still"
    };

    format!(
        "<p>{} {}: <em>{}</em>.</p>",
        notification.status.as_icon(),
        msg,
        notification.status.as_str().to_uppercase()
    )
}

fn format_replicas(notification: &Notification) -> String {
    let replicas = notification
        .replicas
        .iter()
        .map(|replica| replica.split(":").take(2).collect::<Vec<&str>>().join(":"))
        .fold(HashMap::new(), |mut replicas_count, replica| {
            *replicas_count.entry(replica).or_insert(0) += 1;
            replicas_count
        })
        .iter()
        .map(|(service_and_node, count)| {
            format!(
                "<li><code>{}</code>: {} {}</li>",
                service_and_node,
                count,
                notification.status.as_str()
            )
        })
        .collect::<Vec<String>>();

    if replicas.is_empty() {
        "".to_string()
    } else {
        format!("<ul>{}</ul>", replicas.join(""))
    }
}

fn format_status_page(_: &Notification) -> String {
    format!(
        "<p>Status page: {}</p>",
        APP_CONF.branding.page_url.as_str()
    )
}

fn format_time(notification: &Notification) -> String {
    format!("<p>Time: {}</p>", notification.time)
}

fn format_message(notification: &Notification) -> String {
    MATRIX_FORMATTERS
        .iter()
        .fold(String::new(), |mut accumulator, formatter| {
            accumulator.push_str(formatter(notification).as_str());
            accumulator
        })
}
