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
    pub use super::server::start_server;
    pub use super::server::variables::API as server_api;
    pub use super::server::connection::Priviledge;
}
