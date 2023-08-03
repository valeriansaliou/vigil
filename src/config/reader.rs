// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{collections::{hash_set::HashSet, HashMap}, fs, env};
use envsubst::substitute;

use toml;

use super::config::*;
use crate::APP_ARGS;

pub struct ConfigReader;

impl ConfigReader {
    pub fn make() -> Config {
        debug!("reading config file: {}", &APP_ARGS.config);

        let conf = fs::read_to_string(&APP_ARGS.config)
        .expect("cannot find config file");

        // Replace Env
        let env_vars = env::vars().collect::<HashMap<String, String>>();
        let conf = substitute(&conf, &env_vars).expect("cannot substitute environment variables");

        // Parse configuration
        let config = toml::from_str(&conf).expect("syntax error in config file");

        // Validate configuration
        Self::validate(&config);

        config
    }

    fn validate(config: &Config) {
        // Validate all identifiers
        Self::validate_identifiers(config)
    }

    fn validate_identifiers(config: &Config) {
        // Scan for service identifier duplicates
        let mut service_identifiers = HashSet::new();

        for service in config.probe.service.iter() {
            // Service identifier was already previously inserted? (caught a duplicate)
            if service_identifiers.insert(&service.id) == false {
                panic!(
                    "configuration has duplicate service identifier: {}",
                    service.id
                )
            }

            // Scan for node identifier duplicates
            let mut node_identifiers = HashSet::new();

            for node in service.node.iter() {
                // Node identifier was already previously inserted? (caught a duplicate)
                if node_identifiers.insert(&node.id) == false {
                    panic!(
                        "configuration has duplicate node identifier: {} in service: {}",
                        node.id, service.id
                    )
                }
            }
        }
    }
}
