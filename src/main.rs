mod server;
use tokio;
use std::env::args;
#[tokio::main]
async fn main()   {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        panic!("Usage: {} <port>", args[0]);
    }
    if args[0] == "server" {
        server::start_server(args[1].parse().unwrap()).await; // here panic is wanted
    }
    if args[0] == "client" {
        todo!();
        // server::start_client(args[1].parse().unwrap()).await; 
    }
    else {
        println!("Usage: {} <port>", args[0]);
    }

}

