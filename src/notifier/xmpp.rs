// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::RwLock;
use std::time::{Duration, SystemTime};

use libstrophe::{Connection, ConnectionEvent, Context, Stanza};

use super::generic::{GenericNotifier, Notification, DISPATCH_TIMEOUT_SECONDS};
use crate::config::config::ConfigNotify;
use crate::APP_CONF;

pub struct XMPPNotifier;

impl GenericNotifier for XMPPNotifier {
    fn attempt(notify: &ConfigNotify, notification: &Notification) -> Result<(), bool> {
        if let Some(ref xmpp) = notify.xmpp {
            let is_sent = RwLock::new(false);

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
            message.push_str(&format!("URL: {}", APP_CONF.branding.page_url.as_str()));

            debug!("will send XMPP notification with message: {}", &message);

            // Configure connection handler
            let fn_handle =
                |context: &Context, connection: &mut Connection, event: ConnectionEvent| {
                    match event {
                        ConnectionEvent::Connect => {
                            debug!("connected to XMPP account: {}", &xmpp.from);

                            // Acquire UNIX time (used to stamp the message w/ an unique identifier)
                            let now_timestamp = if let Ok(unix_time) =
                                SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
                            {
                                unix_time.as_secs()
                            } else {
                                0
                            };

                            // Send status message
                            let mut message_stanza = Stanza::new_message(
                                Some("chat"),
                                Some(&format!("vigil-{}", now_timestamp)),
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
                        ConnectionEvent::Disconnect(err) => {
                            if let Some(err) = err {
                                error!(
                                    "connection failure to XMPP account: {} ({:?})",
                                    &xmpp.from, err
                                );
                            } else {
                                debug!("disconnected from XMPP account: {}", &xmpp.from);
                            }

                            context.stop();
                        }
                        _ => {}
                    }
                };

            // Configure XMPP connection
            let context = Context::new_with_default_logger();
            let mut connection = Connection::new(context);

            connection.set_jid(&xmpp.from);
            connection.set_pass(&xmpp.xmpp_password);

            connection.set_keepalive(
                Duration::from_secs(DISPATCH_TIMEOUT_SECONDS),
                Duration::from_secs(DISPATCH_TIMEOUT_SECONDS / 2),
            );

            // Connect to XMPP server
            if let Ok(mut connection_context) = connection.connect_client(None, None, &fn_handle) {
                // Enter context
                connection_context.run();

                if *is_sent.read().unwrap() == true {
                    return Ok(());
                }
            }

            return Err(true);
        }

        Err(false)
    }

    fn can_notify(notify: &ConfigNotify, notification: &Notification) -> bool {
        if let Some(ref xmpp_config) = notify.xmpp {
            notification.expected(xmpp_config.reminders_only)
        } else {
            false
        }
    }

    fn name() -> &'static str {
        "xmpp"
    }
}
