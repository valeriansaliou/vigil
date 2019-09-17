// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

//! # OpsGenie notifier module
//!
//! This module has to aim to provide notification using the OpsGenie's alerting [api](https://docs.opsgenie.com/docs/alert-api).
//!
//! To enable this notifier, vigil must be built with the feature `notifier-opsgenie`.
use std::collections::HashMap;

use failure::{format_err, Error, ResultExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::config::{ConfigNotify, ConfigNotifyOpsGenie};
use crate::notifier::generic::{GenericNotifier, Notification};
use crate::prober::status::Status;

// OpsGenieNotifier is the structure that implement the `GenericNotifier` trait.
pub struct OpsGenieNotifier;

impl GenericNotifier for OpsGenieNotifier {
    // Return the name of the integration
    fn name() -> &'static str {
        "opsgenie"
    }

    // Is the notifier can notify? This method return always true because we are handling alerts recovering.
    fn can_notify(_notify: &ConfigNotify, _notification: &Notification) -> bool {
        true
    }

    // Attempt to create, update or close an alert
    fn attempt(notify: &ConfigNotify, notification: &Notification) -> Result<(), bool> {
        // If there is no configuration available for opsgenie, we will exit without done a thing.
        let config = match &notify.opsgenie {
            Some(config) => config,
            None => {
                return Ok(());
            }
        };

        // A replica is a linear combination of the probe_id, node_id and replica_id.
        // This should be enough to be used as an opsgenie alias.
        // Besides, an opsgenie alias is used as a deduplication key.
        let mut has_err = false;
        for replica in &notification.replicas {
            // Try to retrieve the alert.
            let alert = match get(config, replica) {
                Ok(alert) => alert,
                Err(err) => {
                    error!("could not retrieve alert, {}", err);
                    None
                }
            };

            match &alert {
                Some(alert) => {
                    // If the alert exist and the status is not healthy, send a new alert else
                    // if it is a recovery, close gracefully the alert.
                    if &Status::Healthy != notification.status {
                        if let Err(err) =
                            create(config, &AlertCreation::new(config, notification, replica))
                        {
                            error!("could not create alert, {}", err);
                            has_err = true;
                        }
                    } else if let Err(err) = close(config, alert) {
                        error!("could not close alert, {}", err);
                        has_err = true;
                    }
                }
                None => {
                    if &Status::Healthy != notification.status {
                        // The alert do not exist, send a new alert.
                        if let Err(err) =
                            create(config, &AlertCreation::new(config, notification, replica))
                        {
                            error!("could not create alert, {}", err);
                            has_err = true;
                        }
                    }
                }
            }
        }

        if has_err {
            Err(true)
        } else {
            Ok(())
        }
    }
}

#[derive(Deserialize, Serialize)]
struct AlertCreation {
    pub alias: String,
    pub message: String,
    pub actions: Vec<String>,
    pub tags: Vec<String>,
    pub details: HashMap<String, String>,
    pub priority: Option<String>,
    pub user: Option<String>,
}

impl AlertCreation {
    pub fn new(config: &ConfigNotifyOpsGenie, notification: &Notification, replica: &str) -> Self {
        let message = if notification.changed {
            format!(
                "Node '{}' status changed to '{}'",
                replica,
                notification.status.as_str()
            )
        } else {
            format!(
                "Node '{}' status is still '{}'",
                replica,
                notification.status.as_str()
            )
        };

        let mut details = config.details.to_owned();

        details.insert("node".into(), replica.into());

        Self {
            alias: replica.into(),
            message,
            actions: config.actions.to_owned(),
            tags: config.tags.to_owned(),
            details,
            priority: config.priority.to_owned(),
            user: config.user.to_owned(),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct Alert {
    pub id: String,
    pub alias: String,
}

#[derive(Deserialize, Serialize)]
struct Search {
    #[serde(rename = "requestId")]
    pub request_id: String,
    pub data: Vec<Alert>,
}

fn get(config: &ConfigNotifyOpsGenie, replica: &str) -> Result<Option<Alert>, Error> {
    let client = Client::new();
    let mut res = client
        .get(&format!("{}/v2/alerts", config.url))
        .header("Authorization", format!("GenieKey {}", config.key))
        .query(&[("query", &format!("alias: {}", replica))])
        .send()
        .with_context(|err| format!("could not get opsgenie's alert, {}", err))?;

    let status = res.status();
    if !status.is_success() {
        return Err(format_err!(
            "could not get opsgenie's alert, got {}",
            status.as_u16()
        ));
    }

    let search: Search = res
        .json()
        .with_context(|err| format!("could not deserialize search response, {}", err))?;

    for alert in search.data {
        if alert.alias == replica {
            return Ok(Some(alert));
        }
    }

    Ok(None)
}

fn create(config: &ConfigNotifyOpsGenie, alert: &AlertCreation) -> Result<(), Error> {
    let client = Client::new();
    let res = client
        .post(&format!("{}/v2/alerts", config.url))
        .header("Authorization", format!("GenieKey {}", config.key))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(alert).with_context(|err| {
            format_err!("could not serialize alert creation payload, {}", err)
        })?)
        .send()
        .with_context(|err| format!("could not get opsgenie's alert, {}", err))?;

    let status = res.status();
    if !status.is_success() {
        return Err(format_err!(
            "could not create opsgenie's alert, got {}",
            status.as_u16()
        ));
    }

    Ok(())
}

fn close(config: &ConfigNotifyOpsGenie, alert: &Alert) -> Result<(), Error> {
    let client = Client::new();
    let res = client
        .post(&format!("{}/v2/alerts/{}/close", config.url, alert.id))
        .header("Authorization", format!("GenieKey {}", config.key))
        .send()
        .with_context(|err| format!("could not close opsgenie's alert, {}", err))?;

    let status = res.status();
    if !status.is_success() {
        return Err(format_err!(
            "could not close opsgenie's alert, got {}",
            status.as_u16()
        ));
    }

    Ok(())
}
