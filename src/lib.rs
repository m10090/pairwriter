mod server;
mod client;
mod communication;
mod integration_testing;

pub use server::start_server;

pub use client::connect_as_client;

pub use server::variables::API as server_API;


pub use client::API as client_API;

pub use communication::{api_client::ClientApi, api_server::ServerApi};

pub use communication::rpc::RPC;

