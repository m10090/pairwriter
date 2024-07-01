mod connection;
use connection::{*};
use std::env;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;


fn check_message(message: &Message) -> Result<&Message, ()> {
    // to be implemented
    Ok(message)
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let port = String::from("8080");
    let port = args.get(1).unwrap_or(&port);
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(connect_to_server(stream));
    }
}
