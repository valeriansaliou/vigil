// Vigil
//
// Microservices Status Page
// Copyright: 2021, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use actix_web::{
    dev::ServiceRequest,
    guard,
    middleware::{self, normalize::TrailingSlash},
    rt, web, App, Error as ActixError, HttpServer,
};
use actix_web_httpauth::{
    extractors::{
        basic::{BasicAuth, Config as ConfigAuth},
        AuthenticationError,
    },
    middleware::HttpAuthentication,
};
use tera::Tera;

use super::routes;
use crate::APP_CONF;

pub fn run() {
    let mut runtime = rt::System::new("test");

    let templates: String = APP_CONF
        .assets
        .path
        .canonicalize()
        .unwrap()
        .join("templates")
        .join("*")
        .to_str()
        .unwrap()
        .into();

    let tera = Tera::new(&templates).unwrap();
    let middleware_auth = HttpAuthentication::basic(authenticate);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .data(tera.clone())
            .wrap(middleware::NormalizePath::new(TrailingSlash::Trim))
            .service(routes::assets_javascripts)
            .service(routes::assets_stylesheets)
            .service(routes::assets_images)
            .service(routes::assets_fonts)
            .service(routes::badge)
            .service(routes::status_text)
            .service(routes::robots)
            .service(routes::index)
            .data(ConfigAuth::default().realm("Reporter Token"))
            .service(
                web::resource("/reporter/{probe_id}/{node_id}")
                    .wrap(middleware_auth.clone())
                    .guard(guard::Post())
                    .to(routes::reporter_report),
            )
            .service(
                web::resource("/reporter/{probe_id}/{node_id}/{replica_id}")
                    .wrap(middleware_auth.clone())
                    .guard(guard::Delete())
                    .to(routes::reporter_flush),
            )
    })
    .bind(APP_CONF.server.inet)
    .unwrap()
    .run();

    runtime.block_on(server).unwrap()
}

async fn authenticate(
    request: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, ActixError> {
    let password = if let Some(password) = credentials.password() {
        &*password
    } else {
        ""
    };

    if password == APP_CONF.server.reporter_token {
        Ok(request)
    } else {
        let mut error = AuthenticationError::from(
            request
                .app_data::<ConfigAuth>()
                .map(|data| data.clone())
                .unwrap_or_else(ConfigAuth::default),
        );

        *error.status_code_mut() = actix_web::http::StatusCode::FORBIDDEN;

        Err(error.into())
    }
}
