// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde_json;
use std::fs::File;
use std::io::Read;
use toml;

use super::config::*;
use crate::APP_ARGS;

pub struct ConfigReader;

impl ConfigReader {
    pub fn make() -> Config {
        let config_file = &APP_ARGS.config;
        debug!("reading config file: {}", config_file);

        let mut file = File::open(config_file).expect("cannot find config file");
        let mut conf = String::new();

        file.read_to_string(&mut conf)
            .expect("cannot read config file");

        debug!("read config file: {}", config_file);

        if config_file.ends_with(".json") {
            serde_json::from_str(&conf).expect("syntax error in JSON config file")
        } else {
            toml::from_str(&conf).expect("syntax error in TOML config file")
        }
    }
}
