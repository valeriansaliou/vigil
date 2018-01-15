// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket;
use rocket::config::{Config, Environment};

use super::routes;

use APP_CONF;

pub fn run() {
    let config = Config::build(Environment::Production)
        .address(APP_CONF.server.inet.ip().to_string())
        .port(APP_CONF.server.inet.port())
        .workers(2)
        .finalize()
        .unwrap();

    rocket::custom(config, false)
        .mount("/", routes![
            routes::index,
            routes::badge
        ])
        .launch();
}
