// Vigil
//
// Microservices Status Page
// Copyright: 2021, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use actix_files::NamedFile;
use actix_web::{
    dev::ServiceRequest, get, guard, rt, web, web::Data, web::Json, App, Error as ActixError,
    HttpResponse, HttpServer,
};
use actix_web_httpauth::{
    extractors::{
        basic::{BasicAuth, Config as ConfigAuth},
        AuthenticationError,
    },
    middleware::HttpAuthentication,
};
use tera::Tera;

use super::context::{IndexContext, INDEX_CONFIG, INDEX_ENVIRONMENT};
use super::payload::ReporterPayload;
use crate::prober::manager::{run_dispatch_plugins, STORE as PROBER_STORE};
use crate::prober::report::{
    handle_health as handle_health_report, handle_load as handle_load_report, HandleHealthError,
    HandleLoadError,
};
use crate::APP_CONF;

pub async fn reporter(
    web::Path((probe_id, node_id)): web::Path<(String, String)>,
    data: Json<ReporterPayload>,
) -> HttpResponse {
    // Route report to handler (depending on its contents)
    if let Some(ref load) = data.load {
        // Load reports should come for 'push' nodes only
        match handle_load_report(
            &probe_id,
            &node_id,
            &data.replica,
            data.interval,
            load.cpu,
            load.ram,
        ) {
            Ok(forward) => {
                // Trigger a plugins check
                run_dispatch_plugins(&probe_id, &node_id, forward);

                HttpResponse::Ok().finish()
            }
            Err(HandleLoadError::InvalidLoad) => HttpResponse::BadRequest().finish(),
            Err(HandleLoadError::WrongMode) => HttpResponse::PreconditionFailed().finish(),
            Err(HandleLoadError::NotFound) => HttpResponse::NotFound().finish(),
        }
    } else if let Some(ref health) = data.health {
        // Health reports should come for 'local' nodes only
        match handle_health_report(&probe_id, &node_id, &data.replica, data.interval, health) {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(HandleHealthError::WrongMode) => HttpResponse::PreconditionFailed().finish(),
            Err(HandleHealthError::NotFound) => HttpResponse::NotFound().finish(),
        }
    } else {
        // Report contents is invalid
        HttpResponse::BadRequest().finish()
    }
}

#[get("/robots.txt")]
pub async fn robots() -> Option<NamedFile> {
    NamedFile::open(APP_CONF.assets.path.join("public").join("robots.txt")).ok()
}

#[get("/status/text")]
pub async fn status_text() -> &'static str {
    &PROBER_STORE.read().unwrap().states.status.as_str()
}

#[get("/badge/{kind}")]
pub async fn badge(web::Path(kind): web::Path<String>) -> Option<NamedFile> {
    // Notice acquire lock in a block to release it ASAP (ie. before OS access to file)
    let status = { &PROBER_STORE.read().unwrap().states.status.as_str() };

    NamedFile::open(
        APP_CONF
            .assets
            .path
            .join("images")
            .join("badges")
            .join(format!("{}-{}-default.svg", kind, status)),
    )
    .ok()
}

#[get("/assets/fonts/{folder}/{file}")]
pub async fn assets_fonts(
    web::Path((folder, file)): web::Path<(String, String)>,
) -> Option<NamedFile> {
    NamedFile::open(APP_CONF.assets.path.join("fonts").join(folder).join(file)).ok()
}

#[get("/assets/images/{folder}/{file}")]
pub async fn assets_images(
    web::Path((folder, file)): web::Path<(String, String)>,
) -> Option<NamedFile> {
    NamedFile::open(APP_CONF.assets.path.join("images").join(folder).join(file)).ok()
}

#[get("/assets/stylesheets/{file}")]
pub async fn assets_stylesheets(web::Path(file): web::Path<String>) -> Option<NamedFile> {
    NamedFile::open(APP_CONF.assets.path.join("stylesheets").join(file)).ok()
}

#[get("/assets/javascripts/{file}")]
pub async fn assets_javascripts(web::Path(file): web::Path<String>) -> Option<NamedFile> {
    NamedFile::open(APP_CONF.assets.path.join("javascripts").join(file)).ok()
}

#[get("/")]
pub fn index(tera: Data<Tera>) -> HttpResponse {
    // Notice acquire lock in a block to release it ASAP (ie. before template renders)
    let context = {
        IndexContext {
            states: &PROBER_STORE.read().unwrap().states,
            environment: &*INDEX_ENVIRONMENT,
            config: &*INDEX_CONFIG,
        }
    };
    let render = tera.render(
        "index.tera",
        &tera::Context::from_serialize(context).unwrap(),
    );
    if let Ok(s) = render {
        HttpResponse::Ok().content_type("text/html").body(s)
    } else {
        HttpResponse::InternalServerError().body(format!("Template Error {:?}", render))
    }
}

async fn authenticate(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, ActixError> {
    let config = req
        .app_data::<ConfigAuth>()
        .map(|data| data.clone())
        .unwrap_or_else(ConfigAuth::default);
    if let Some(password) = credentials.password() {
        if *password == APP_CONF.server.reporter_token {
            Ok(req)
        } else {
            let mut error = AuthenticationError::from(config);
            *error.status_code_mut() = actix_web::http::StatusCode::FORBIDDEN;
            Err(error.into())
        }
    } else {
        Err(AuthenticationError::from(config).into())
    }
}

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
            .data(tera.clone())
            .service(assets_javascripts)
            .service(assets_stylesheets)
            .service(assets_images)
            .service(assets_fonts)
            .service(badge)
            .service(status_text)
            .service(robots)
            .service(index)
            .data(ConfigAuth::default().realm("Reporter Token"))
            .service(
                web::resource("/reporter/{probe_id}/{node_id}")
                    .wrap(middleware_auth.clone())
                    .guard(guard::Post())
                    .to(reporter),
            )
    })
    .bind(APP_CONF.server.inet)
    .unwrap()
    .run();

    runtime.block_on(server).unwrap()
}
