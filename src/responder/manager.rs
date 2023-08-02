// Vigil
//
// Microservices Status Page
// Copyright: 2021, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use actix_web::{
    dev::ServiceRequest,
    guard,
    middleware::{self},
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
    let runtime = rt::System::new();

    // Prepare templating engine
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

    // Prepare authentication middlewares
    let (middleware_reporter_auth, middleware_manager_auth) = (
        HttpAuthentication::basic(authenticate_reporter),
        HttpAuthentication::basic(authenticate_manager),
    );

    // Start the HTTP server
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .wrap(middleware::NormalizePath::default())
            .service(routes::assets_javascripts)
            .service(routes::assets_stylesheets)
            .service(routes::assets_images)
            .service(routes::assets_fonts)
            .service(routes::badge)
            .service(routes::status_text)
            .service(routes::robots)
            .service(routes::index)
            .app_data(ConfigAuth::default().realm("Reporter Token"))
            .service(
                web::resource("/reporter/{probe_id}/{node_id}")
                    .wrap(middleware_reporter_auth.clone())
                    .guard(guard::Post())
                    .to(routes::reporter_report),
            )
            .service(
                web::resource("/reporter/{probe_id}/{node_id}/{replica_id}")
                    .wrap(middleware_reporter_auth.clone())
                    .guard(guard::Delete())
                    .to(routes::reporter_flush),
            )
            .service(
                web::resource("/manager/announcements")
                    .wrap(middleware_manager_auth.clone())
                    .guard(guard::Get())
                    .to(routes::manager_announcements),
            )
            .service(
                web::resource("/manager/announcement")
                    .wrap(middleware_manager_auth.clone())
                    .guard(guard::Post())
                    .to(routes::manager_announcement_insert),
            )
            .service(
                web::resource("/manager/announcement/{announcement_id}")
                    .wrap(middleware_manager_auth.clone())
                    .guard(guard::Delete())
                    .to(routes::manager_announcement_retract),
            )
            .service(
                web::resource("/manager/prober/alerts")
                    .wrap(middleware_manager_auth.clone())
                    .guard(guard::Get())
                    .to(routes::manager_prober_alerts),
            )
            .service(
                web::resource("/manager/prober/alerts/ignored")
                    .wrap(middleware_manager_auth.clone())
                    .guard(guard::Get())
                    .to(routes::manager_prober_alerts_ignored_resolve),
            )
            .service(
                web::resource("/manager/prober/alerts/ignored")
                    .wrap(middleware_manager_auth.clone())
                    .guard(guard::Put())
                    .to(routes::manager_prober_alerts_ignored_update),
            )
    })
    .workers(APP_CONF.server.workers)
    .bind(APP_CONF.server.inet)
    .unwrap()
    .run();

    runtime.block_on(server).unwrap()
}

fn authenticate(
    request: ServiceRequest,
    credentials: BasicAuth,
    token: &str,
) -> Result<ServiceRequest, (ActixError, ServiceRequest)> {
    let password = if let Some(password) = credentials.password() {
        &*password
    } else {
        ""
    };

    if password == token {
        Ok(request)
    } else {
        let mut error = AuthenticationError::from(
            request
                .app_data::<ConfigAuth>()
                .map(|data| data.clone())
                .unwrap_or_else(ConfigAuth::default),
        );

        *error.status_code_mut() = actix_web::http::StatusCode::FORBIDDEN;

        Err((error.into(), request))
    }
}

async fn authenticate_reporter(
    request: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, (ActixError, ServiceRequest)> {
    authenticate(request, credentials, &APP_CONF.server.reporter_token)
}

async fn authenticate_manager(
    request: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, (ActixError, ServiceRequest)> {
    authenticate(request, credentials, &APP_CONF.server.manager_token)
}
