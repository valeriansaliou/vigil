// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket;

use super::routes;

pub fn run() {
    // TODO: pass listen in config.cfg configuration, do not read Rocket.toml

    rocket::ignite()
        .mount("/", routes![
            routes::index,
            routes::badge
        ])
        .launch();
}
