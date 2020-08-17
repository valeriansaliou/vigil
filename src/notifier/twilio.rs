// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;
use std::time::Duration;

use reqwest::blocking::Client;

use super::generic::{GenericNotifier, Notification, DISPATCH_TIMEOUT_SECONDS};
use crate::config::config::ConfigNotify;
use crate::APP_CONF;

static TEXT_MESSAGE_TRUNCATED_INDICATOR: &'static str = "[..]";

const TEXT_MESSAGE_MAXIMUM_LENGTH: usize = 1000;

lazy_static! {
    static ref TWILIO_HTTP_CLIENT: Client = Client::builder()
        .timeout(Duration::from_secs(DISPATCH_TIMEOUT_SECONDS))
        .gzip(true)
        .build()
        .unwrap();
}

pub struct TwilioNotifier;

impl GenericNotifier for TwilioNotifier {
    fn attempt(notify: &ConfigNotify, notification: &Notification) -> Result<(), bool> {
        if let Some(ref twilio) = notify.twilio {
            // Build up the message text
            let mut message = String::new();

            if notification.startup == true {
                message.push_str("Startup alert for: ");
            } else if notification.changed == false {
                message.push_str("Reminder for: ");
            }

            message.push_str(&format!("{}\n", APP_CONF.branding.page_title));
            message.push_str("\n");
            message.push_str(&format!("Status: {:?}\n", notification.status));
            message.push_str(&format!("Nodes: {}\n", &notification.replicas.join(", ")));
            message.push_str(&format!("Time: {}\n", &notification.time));

            // Trim down message to a maximum length? (most SMS receivers and networks support \
            //   up to 1600 characters by re-building message segments)
            if message.len() > TEXT_MESSAGE_MAXIMUM_LENGTH {
                debug!(
                    "message for Twilio notification is too long, trimming to length: {}",
                    TEXT_MESSAGE_MAXIMUM_LENGTH
                );

                message
                    .truncate(TEXT_MESSAGE_MAXIMUM_LENGTH - TEXT_MESSAGE_TRUNCATED_INDICATOR.len());

                message.push_str(TEXT_MESSAGE_TRUNCATED_INDICATOR);
            }

            debug!("will send Twilio notification with message: {}", &message);

            let mut has_sub_delivery_failure = false;

            for to_number in &twilio.to {
                // Build form parameters
                let mut params = HashMap::new();

                params.insert("MessagingServiceSid", &twilio.service_sid);
                params.insert("To", to_number);
                params.insert("Body", &message);

                // Submit message to Twilio
                let response = TWILIO_HTTP_CLIENT
                    .post(&generate_api_url(&twilio.account_sid))
                    .basic_auth(
                        twilio.account_sid.as_str(),
                        Some(twilio.auth_token.as_str()),
                    )
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
        if let Some(ref twilio_config) = notify.twilio {
            notification.expected(twilio_config.reminders_only)
        } else {
            false
        }
    }

    fn name() -> &'static str {
        "twilio"
    }
}

fn generate_api_url(account_sid: &str) -> String {
    format!(
        "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
        account_sid
    )
}
