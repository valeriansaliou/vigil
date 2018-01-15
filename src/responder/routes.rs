// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[get("/")]
fn index() -> &'static str {
    "TODO"
}

#[get("/badge/<size>")]
fn badge(size: u16) -> String {
    format!("TODO: size={}", size)
}
