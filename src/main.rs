#![allow(dead_code)]

mod client;
mod communication;
mod integration_testing;
mod server;
use std::env::args;


#[tokio::main]
async fn main() {
    let args: Vec<String> = args().collect();

    if args.len() < 2 {
        panic!("Usage: (client|server) <port>");
    }
    match args[1].as_str() {
        "server" => {
            let port = args[2].parse().unwrap();
            drop(args);
            #[cfg(feature = "integration_testing_server")]
            {
                // this for integration testing
                use tokio;
                tokio::spawn(integration_testing::run_test());
            }
            server::start_server(port).await; // here panic is wanted
        }
        "client" => {
            if args.len() < 4 {
                panic!("Usage: (client|server) <port> <username>");
            }
            let url = args.get(2).unwrap().to_string();
            let username = args.get(3).unwrap().to_string();
            drop(args);
            #[cfg(feature = "integration_testing_client")]
            {
                // this for integration testing
                use tokio;
                tokio::spawn(integration_testing::run_test());
            }
            client::connect_as_client(url, username).await;
        }
        _ => {
            panic!("Usage: (client|server) <port>")
        }
    }
}
