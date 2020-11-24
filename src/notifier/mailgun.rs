// Vigil
//
// Microservices Status Page
// Copyright: 2019, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::Duration;
use std::collections::HashMap;

use reqwest::blocking::Client;

use super::generic::{GenericNotifier, Notification, DISPATCH_TIMEOUT_SECONDS};
use crate::config::config::ConfigNotify;
// use crate::prober::status::Status;
use crate::APP_CONF;

lazy_static! {
    static ref MAILGUN_HTTP_CLIENT: Client = Client::builder()
        .timeout(Duration::from_secs(DISPATCH_TIMEOUT_SECONDS))
        .gzip(true)
        .build()
        .unwrap();
}

pub struct MailgunNotifier;

impl GenericNotifier for MailgunNotifier {
    fn attempt(notify: &ConfigNotify, notification: &Notification) -> Result<(), bool> {
        if let Some(ref mailgun) = notify.mailgun {
            
            let nodes_label = notification.replicas.join("\n, ");
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

            // Submit payload to Web Hooks
            let mut sender = String::new();
                sender.push_str(&format!("KKiaPay Status : {}", mailgun.from));
            let mut subject = String::new();
                subject.push_str(&format!(
                    "{}",
                    notification.status.as_str().to_uppercase()
                ));
            let mut has_sub_delivery_failure = false;

            for to_email in &mailgun.to {
                // Build form parameters
                let mut params = HashMap::new();
                params.insert("from", &sender);
                params.insert("to", to_email);
                params.insert("subject", &subject);
                params.insert("text", &message);     

                let response = MAILGUN_HTTP_CLIENT
                .post(mailgun.api_url.as_str())
                .basic_auth("api", Some(&mailgun.api_key))
                .form(&params)
                .send();

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
            

            // return Err(true);
        }

        Err(false)
    }

    fn can_notify(notify: &ConfigNotify, _: &Notification) -> bool {
        notify.mailgun.is_some()
    }

    fn name() -> &'static str {
        "mailgun"
    }
}
