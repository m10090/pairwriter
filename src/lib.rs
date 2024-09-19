#![allow(dead_code)]

mod client;
mod communication;
mod server;

#[cfg(feature = "integration_testing")]
mod integration_testing;

pub mod prelude {
    pub use super::client::api_client::ClientApi;
    pub use super::client::connect_as_client;
    pub use super::client::API as client_api;
    pub use super::communication::rpc::RPC;
    pub use super::server::api_server::ServerApi;
    pub use super::server::connection::Priviledge;
    pub use super::server::start_server;
    pub use super::server::variables::API as server_api;
}

pub mod server_import {
    pub use super::prelude::{server_api, start_server, Priviledge, ServerApi};
}

pub mod client_import {
    pub use super::prelude::{client_api, connect_as_client, ClientApi, Priviledge};
}

// env you need to define `LOGFILE`, `SERVER_USERNAME` 
