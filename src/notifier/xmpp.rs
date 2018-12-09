// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::Duration;
use std::sync::RwLock;
use time;

use libstrophe::{Connection, ConnectionEvent, Context, Stanza};
use libstrophe::error::StreamError;

use super::generic::{DISPATCH_TIMEOUT_SECONDS, Notification, GenericNotifier};
use config::config::ConfigNotify;
use APP_CONF;

pub struct XMPPNotifier;

impl GenericNotifier for XMPPNotifier {
    fn attempt(notify: &ConfigNotify, notification: &Notification) -> Result<(), bool> {
        if let Some(ref xmpp) = notify.xmpp {
            let is_sent = RwLock::new(false);

            // Build up the message text
            let mut message = String::new();

            if notification.changed == false {
                message.push_str("Reminder for: ");
            }

            message.push_str(&format!("{}\n", APP_CONF.branding.page_title));
            message.push_str("\n");
            message.push_str(&format!("Status: {:?}\n", notification.status));
            message.push_str(&format!("Nodes: {}\n", &notification.replicas.join(", ")));
            message.push_str(&format!("Time: {}\n", &notification.time));
            message.push_str(&format!("URL: {}", APP_CONF.branding.page_url.as_str()));

            debug!("will send XMPP notification with message: {}", &message);

            // Configure connection handler
            let fn_handle = |connection: &mut Connection,
                             event: ConnectionEvent,
                             _error: i32,
                             _stream_error: Option<&StreamError>| {
                let context = connection.context();

                match event {
                    ConnectionEvent::XMPP_CONN_CONNECT => {
                        debug!("connected to XMPP account: {}", &xmpp.from);

                        // Send status message
                        let mut message_stanza =
                            Stanza::new_message(
                                &context,
                                Some("chat"),
                                Some(&format!("vigil-{}", time::now().to_timespec().sec)),
                                Some(&xmpp.to),
                            );

                        if message_stanza.set_body(&message).is_ok() == true {
                            connection.send(&message_stanza);

                            {
                                let mut is_sent_value = is_sent.write().unwrap();

                                *is_sent_value = true;
                            }
                        }

                        // Disconnect immediately
                        connection.disconnect();
                    }
                    ConnectionEvent::XMPP_CONN_DISCONNECT |
                    ConnectionEvent::XMPP_CONN_FAIL => {
                        debug!(
                            "disconnected from XMPP account: {} ({:?})",
                            &xmpp.from,
                            event
                        );

                        context.stop();
                    }
                    _ => {}
                }
            };

            // Configure XMPP connection
            let context = Context::new_with_default_logger();
            let mut connection = Connection::new(context.clone());

            connection.set_jid(&xmpp.from);
            connection.set_pass(&xmpp.xmpp_password);

            connection.set_keepalive(
                Duration::from_secs(DISPATCH_TIMEOUT_SECONDS),
                Duration::from_secs(DISPATCH_TIMEOUT_SECONDS / 2),
            );

            // Connect to XMPP server
            if connection.connect_client(None, None, &fn_handle).is_ok() == true {
                // Enter context
                context.run();

                if *is_sent.read().unwrap() == true {
                    return Ok(());
                }
            }

            return Err(true);
        }

        Err(false)
    }

    fn is_enabled(notify: &ConfigNotify) -> bool {
        notify.xmpp.is_some()
    }

    fn name() -> &'static str {
        "xmpp"
    }
}
