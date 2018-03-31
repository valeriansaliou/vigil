// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::Duration;

use libstrophe::{self, Connection, ConnectionEvent, ConnectionFlags, Context, Stanza};
use libstrophe::error::StreamError;

use super::generic::{DISPATCH_TIMEOUT_SECONDS, Notification, GenericNotifier};
use config::config::ConfigNotify;

pub struct XMPPNotifier;

impl GenericNotifier for XMPPNotifier {
    fn attempt(notify: &ConfigNotify, notification: &Notification) -> Result<(), bool> {
        if let Some(ref xmpp) = notify.xmpp {
            let mut is_sent = false;

            // Configure connection handler
            let handler = move |connection: &mut Connection,
                                event: ConnectionEvent,
                                _error: i32,
                                _stream_error: Option<&StreamError>| {
                // TODO
                error!("==> XMPP: event");

                match event {
                    ConnectionEvent::XMPP_CONN_CONNECT => {
                        // TODO
                        error!("==> XMPP: connect");

                        // Send status message
                        // Stanza::new_message(context, Some(type), None, Some(to))

                        is_sent = true;

                        // Disconnect immediately
                        connection.disconnect();
                    }
                    ConnectionEvent::XMPP_CONN_DISCONNECT |
                    ConnectionEvent::XMPP_CONN_FAIL => {
                        // TODO
                        error!("==> XMPP: disconnect or fail");

                        connection.context().stop();
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

            let mut flags = ConnectionFlags::empty();

            flags.set(ConnectionFlags::DISABLE_TLS, false);
            flags.set(ConnectionFlags::MANDATORY_TLS, xmpp.xmpp_encrypt);
            flags.set(ConnectionFlags::LEGACY_SSL, false);

            connection.set_flags(flags).ok();

            // Connect to XMPP server
            if connection.connect_client(None, None, &handler).is_ok() == true {
                // Enter context
                context.run();

                // Left context
                libstrophe::shutdown();

                if is_sent == true {
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
