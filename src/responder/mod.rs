// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[cfg(feature = "web-rocket")]
mod asset_file;
mod context;
#[cfg(feature = "web-rocket")]
mod reporter_guard;
#[cfg(feature = "web-rocket")]
mod routes;

#[cfg(feature = "web-rocket")]
pub mod manager;
