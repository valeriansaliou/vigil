// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;

use rocket;
use rocket::config::{Config, Environment};
use rocket_contrib::Template;

use super::routes;

use APP_CONF;

pub fn run() {
    // Build Rocket configuration
    let mut config = Config::build(Environment::Production)
        .address(APP_CONF.server.inet.ip().to_string())
        .port(APP_CONF.server.inet.port())
        .workers(2)
        .finalize()
        .unwrap();

    // Append extra options
    let mut extras = HashMap::new();

    extras.insert(
        "template_dir".to_string(),
        APP_CONF
            .assets
            .path
            .join("./templates")
            .to_str()
            .unwrap()
            .into(),
    );

    config.set_extras(extras);

    // Build and run Rocket instance
    rocket::custom(config, false)
        .mount(
            "/",
            routes![
            routes::index,
            routes::reporter,
            routes::robots,
            routes::badge,
            routes::assets_fonts,
            routes::assets_images,
            routes::assets_stylesheets,
        ],
        )
        .attach(Template::fairing())
        .launch();
}
