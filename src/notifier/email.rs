// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::Duration;

use lettre::message::{Mailbox, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::transport::smtp::SmtpTransport;
use lettre::{Address, Transport};

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
            let email_message = Message::builder()
                .to(Mailbox::new(
                    None,
                    email_config.to.parse::<Address>().or(Err(true))?,
                ))
                .from(Mailbox::new(
                    Some(APP_CONF.branding.page_title.to_owned()),
                    email_config.from.parse::<Address>().or(Err(true))?,
                ))
                .subject(format!(
                    "{} | {}",
                    notification.status.as_str().to_uppercase(),
                    &nodes_label
                ))
                .body(message)
                .or(Err(true))?;

            // Deliver the message
            return acquire_transport(
                &email_config.smtp_host,
                email_config.smtp_port,
                email_config.smtp_username.to_owned(),
                email_config.smtp_password.to_owned(),
                email_config.smtp_encrypt,
            )
            .map(|transport| transport.send(&email_message))
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
    // Acquire credentials (if any)
    let credentials = if let (Some(smtp_username_value), Some(smtp_password_value)) =
        (smtp_username, smtp_password)
    {
        Some(Credentials::new(
            smtp_username_value.to_owned(),
            smtp_password_value.to_owned(),
        ))
    } else {
        None
    };

    // Acquire TLS wrapper (may fail)
    let tls_wrapper = if let Ok(tls_parameters) = TlsParameters::new(smtp_host.into()) {
        if smtp_encrypt == true {
            Ok(Tls::Required(tls_parameters))
        } else {
            Ok(Tls::Opportunistic(tls_parameters))
        }
    } else {
        Err(())
    };

    // Build transport
    let mut mailer = SmtpTransport::builder_dangerous(smtp_host)
        .port(smtp_port)
        .tls(tls_wrapper.or(Err(()))?)
        .timeout(Some(Duration::from_secs(DISPATCH_TIMEOUT_SECONDS)));

    if let Some(credentials_value) = credentials {
        mailer = mailer.credentials(credentials_value);
    }

    Ok(mailer.build())
}
