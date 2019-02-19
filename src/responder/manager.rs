// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;

use rocket;
use rocket::config::{Config, Environment};
use rocket_contrib::templates::Template;

use super::routes;

use crate::APP_CONF;

pub fn run() {
    // Build Rocket configuration
    let mut config = Config::build(Environment::Production)
        .address(APP_CONF.server.inet.ip().to_string())
        .port(APP_CONF.server.inet.port())
        .workers(APP_CONF.server.workers)
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
    rocket::custom(config)
        .mount(
            "/",
            routes![
                routes::index,
                routes::reporter,
                routes::robots,
                routes::status_text,
                routes::badge,
                routes::assets_fonts,
                routes::assets_images,
                routes::assets_stylesheets,
                routes::assets_javascripts,
            ],
        )
        .attach(Template::fairing())
        .launch();
}
