// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::Duration;

use lettre::message::{Mailbox, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::transport::smtp::{Error as SmtpError, SmtpTransport};
use lettre::{Address, Transport};

use super::generic::{GenericNotifier, Notification, DISPATCH_TIMEOUT_SECONDS};
use crate::config::config::ConfigNotify;
use crate::APP_CONF;

#[derive(Default)]
pub struct EmailNotifier;

impl GenericNotifier for EmailNotifier {
    fn attempt(notify: &ConfigNotify, notification: &Notification) -> Result<(), bool> {
        let email_config = match &notify.email {
            Some(cfg) => cfg,
            None => return Err(false),
        };

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

        // Create the transport if not present
        let transport = match acquire_transport(
            &email_config.smtp_host,
            email_config.smtp_port,
            email_config.smtp_username.to_owned(),
            email_config.smtp_password.to_owned(),
            email_config.smtp_encrypt,
        ) {
            Ok(t) => t,
            Err(e) => {
                error!("failed to build email transport: {e}");
                return Err(true);
            }
        };

        // Deliver the message
        if let Err(e) = transport.send(&email_message) {
            error!("failed to send email: {e}");
            return Err(true);
        }

        Ok(())
    }

    fn can_notify(notify: &ConfigNotify, notification: &Notification) -> bool {
        notify
            .email
            .as_ref()
            .map_or(false, |cfg| notification.expected(cfg.reminders_only))
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
) -> Result<SmtpTransport, SmtpError> {
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
    let tls_wrapper = match TlsParameters::new(smtp_host.into()) {
        Ok(p) if smtp_encrypt => Tls::Required(p),
        Ok(p) => Tls::Opportunistic(p),
        Err(e) => return Err(e),
    };

    // Build transport
    let mut mailer = SmtpTransport::builder_dangerous(smtp_host)
        .port(smtp_port)
        .tls(tls_wrapper)
        .timeout(Some(Duration::from_secs(DISPATCH_TIMEOUT_SECONDS)));

    if let Some(credentials_value) = credentials {
        mailer = mailer.credentials(credentials_value);
    }

    Ok(mailer.build())
}
