mod server;
mod client;
mod communication;
mod integration_testing;


pub mod prelude {
    pub use super::server::variables::API as server_api;
    pub use super::client::API as client_api;
    pub use super::client::api_client::ClientApi;
    pub use super::communication::rpc::RPC;
    pub use super::server::api_server::ServerApi;
}
