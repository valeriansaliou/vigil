// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use reqwest::{Client, StatusCode};

use super::generic::{Notification, GenericNotifier};
use APP_CONF;

// TODO: enforce a timeout

pub struct SlackNotifier;

impl GenericNotifier for SlackNotifier {
    fn dispatch(notification: &Notification) -> Result<(), bool> {
        if Self::is_enabled() == true {
            if let Some(ref slack) = APP_CONF.notify.slack {
                debug!(
                    "dispatch slack notification for status: {:?} and replicas: {:?}",
                    notification.status, notification.replicas
                );

                // let response = Client::new().post(slack.hook_url).json(payload).send()?;

                // if response.status() == StatusCode::Ok {
                //     return Ok(());
                // } else {
                    return Err(true);
                // }
            }
        }

        Err(false)
    }

    fn is_enabled() -> bool {
        APP_CONF.notify.slack.is_some()
    }
}
