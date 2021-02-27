// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::Duration;

use log::{debug, error};
use lettre::smtp::authentication::Credentials;
use lettre::smtp::client::net::ClientTlsParameters;
use lettre::smtp::{ClientSecurity, ConnectionReuseParameters, SmtpClient, SmtpTransport};
use lettre::Transport;
use lettre_email::EmailBuilder;
use native_tls::TlsConnector;

use super::generic::{GenericNotifier, Notification, DISPATCH_TIMEOUT_SECONDS};
use crate::config::config::ConfigNotify;
use crate::APP_CONF;

pub struct EmailNotifier;

impl GenericNotifier for EmailNotifier {
    fn attempt(notify: &ConfigNotify, notification: &Notification) -> Result<(), bool> {
        if let Some(ref email_config) = notify.email {
            let nodes_label = notification.replicas.join(", ");

            // Build up the message text
            let mut message = String::new();

            if notification.startup == true {
                message.push_str(&format!(
                    "Status startup alert from: {}\n",
                    APP_CONF.branding.page_title
                ));
            } else if notification.changed == true {
                message.push_str(&format!(
                    "Status change report from: {}\n",
                    APP_CONF.branding.page_title
                ));
            } else {
                message.push_str(&format!(
                    "Status unchanged reminder from: {}\n",
                    APP_CONF.branding.page_title
                ));
            }

            message.push_str("\n--\n");
            message.push_str(&format!("Status: {:?}\n", notification.status));
            message.push_str(&format!("Nodes: {}\n", &nodes_label));
            message.push_str(&format!("Time: {}\n", &notification.time));
            message.push_str(&format!("URL: {}", APP_CONF.branding.page_url.as_str()));

            message.push_str("\n--\n");
            message.push_str("\n");
            message.push_str("To unsubscribe, please edit your status page configuration.");

            debug!("will send email notification with message: {}", &message);

            // Build up the email
            let email_message = EmailBuilder::new()
                .to(email_config.to.as_str())
                .from((
                    email_config.from.as_str(),
                    APP_CONF.branding.page_title.as_str(),
                ))
                .subject(format!(
                    "{} | {}",
                    notification.status.as_str().to_uppercase(),
                    &nodes_label
                ))
                .text(message)
                .build()
                .or(Err(true))?;

            // Deliver the message
            return acquire_transport(
                &email_config.smtp_host,
                email_config.smtp_port,
                email_config.smtp_username.to_owned(),
                email_config.smtp_password.to_owned(),
                email_config.smtp_encrypt,
            )
            .map(|mut transport| transport.send(email_message.into()))
            .and(Ok(()))
            .or(Err(true));
        }

        Err(false)
    }

    fn can_notify(notify: &ConfigNotify, notification: &Notification) -> bool {
        if let Some(ref email_config) = notify.email {
            notification.expected(email_config.reminders_only)
        } else {
            false
        }
    }

    fn name() -> &'static str {
        "email"
    }
}

fn acquire_transport(
    smtp_host: &str,
    smtp_port: u16,
    smtp_username: Option<String>,
    smtp_password: Option<String>,
    smtp_encrypt: bool,
) -> Result<SmtpTransport, ()> {
    let mut security = ClientSecurity::None;

    if smtp_encrypt == true {
        let connector_builder = TlsConnector::builder();
        if let Ok(connector) = connector_builder.build() {
            security = ClientSecurity::Required(ClientTlsParameters {
                connector: connector,
                domain: smtp_host.to_string(),
            });
        }

        // Do not deliver email if TLS context cannot be acquired (prevents unencrypted emails \
        //   to be sent)
        if let ClientSecurity::None = security {
            error!("could not build smtp encrypted connector");

            return Err(());
        }
    }

    match SmtpClient::new(format!("{}:{}", smtp_host, smtp_port), security) {
        Ok(transport) => {
            let mut transport_builder = transport
                .timeout(Some(Duration::from_secs(DISPATCH_TIMEOUT_SECONDS)))
                .connection_reuse(ConnectionReuseParameters::NoReuse);

            match (smtp_username, smtp_password) {
                (Some(smtp_username_value), Some(smtp_password_value)) => {
                    transport_builder = transport_builder
                        .credentials(Credentials::new(smtp_username_value, smtp_password_value));
                }
                _ => {}
            }

            Ok(transport_builder.transport())
        }
        Err(err) => {
            error!("could not acquire smtp transport: {}", err);

            Err(())
        }
    }
}
