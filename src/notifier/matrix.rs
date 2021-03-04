// Vigil
//
// Microservices Status Page
// Copyright: 2021, Valerian Saliou <valerian@valeriansaliou.name>
// Copyright: 2021, Enrico Risa https://github.com/wolf4ood
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{collections::HashMap, convert::TryFrom};

use lazy_static::lazy_static;
use log::debug;
use matrix_sdk::{
    events::{room::message::MessageEventContent, AnyMessageEventContent},
    identifiers::{RoomId, UserId},
    Client, ClientConfig, Session,
};

use super::generic::{GenericNotifier, Notification};
use crate::config::config::{ConfigNotify, ConfigNotifyMatrix};
use crate::APP_CONF;

lazy_static! {
    static ref TOKIO_RUNTIME: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();
    static ref FORMATTERS: Vec<fn(&Notification) -> String> = vec![
        format_status,
        format_replicas,
        format_status_page,
        format_time
    ];
}

pub struct MatrixNotifier;

#[derive(Debug)]
pub struct MatrixNotifierError(String);

impl GenericNotifier for MatrixNotifier {
    fn attempt(notify: &ConfigNotify, notification: &Notification) -> Result<(), bool> {
        if let Some(matrix) = &notify.matrix {
            let response: Result<(), Box<dyn std::error::Error>> = TOKIO_RUNTIME.block_on(async {
                let client = setup_client(matrix).await?;

                send_message(client, matrix, format_message(notification)).await?;

                Ok(())
            });

            return response.map_err(|_| true);
        }

        Ok(())
    }

    fn can_notify(notify: &ConfigNotify, notification: &Notification) -> bool {
        if let Some(ref matrix_cfg) = notify.matrix {
            notification.expected(matrix_cfg.reminders_only)
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
                "<li> <code>{}</code>: {} {}</li>",
                service_and_node,
                count,
                notification.status.as_str()
            )
        })
        .collect::<Vec<String>>()
        .join("");

    format!("<ul>{}</ul>", replicas)
}

fn format_status_page(_: &Notification) -> String {
    format!(
        "<p>Status page: {}</p>",
        APP_CONF.branding.page_url.as_str()
    )
}

fn format_time(notification: &Notification) -> String {
    format!("<p>Time : {}</p>", notification.time)
}

fn format_message(notification: &Notification) -> String {
    FORMATTERS.iter().fold(String::new(), |mut acc, formatter| {
        acc.push_str(formatter(notification).as_str());
        acc
    })
}

async fn setup_client(matrix: &ConfigNotifyMatrix) -> Result<Client, Box<dyn std::error::Error>> {
    let client_config = ClientConfig::default();

    let domain = matrix.homeserver_url.domain().unwrap_or_default();

    let client = Client::new_with_config(matrix.homeserver_url.0.as_str(), client_config)?;

    match (matrix.password.as_ref(), matrix.access_token.as_ref()) {
        (_, Some(access_token)) => {
            client
                .restore_login(Session {
                    access_token: access_token.clone(),
                    user_id: UserId::try_from(format!("@{}:{}", matrix.username.as_str(), domain))?,
                    device_id: matrix.device_id.as_str().into(),
                })
                .await?;
        }
        (Some(password), _) => {
            client
                .login(
                    matrix.username.as_str(),
                    password,
                    Some(matrix.device_id.as_str()),
                    None,
                )
                .await?;
        }

        _ => {
            return Err(Box::new(MatrixNotifierError(String::from(
                "missing password or access_token",
            ))))
        }
    };

    Ok(client)
}

async fn send_message(
    client: Client,
    matrix: &ConfigNotifyMatrix,
    message: String,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("will send Matrix notification with message: {}", &message);

    let content = AnyMessageEventContent::RoomMessage(MessageEventContent::text_html(
        message.as_str(),
        message.as_str(),
    ));

    client
        .room_send(&RoomId::try_from(matrix.room_id.as_str())?, content, None)
        .await?;

    Ok(())
}

impl std::fmt::Display for MatrixNotifierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MatrixNotifier error : {}", self.0)
    }
}

impl std::error::Error for MatrixNotifierError {}
