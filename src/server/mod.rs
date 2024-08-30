use futures::{SinkExt as _, StreamExt as _};
use lazy_static::lazy_static;
use std::collections::HashMap;
use tokio::{
    net::TcpListener,
    sync::{mpsc, Mutex},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use variables::*;

pub async fn start_server(port: u16) {
    // main point
    let url = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&url).await.unwrap(); // panic is needed
                                                           // when there is a connection made to the server
    tokio::spawn(messageing::handle_messages());
    while let Ok((socket, _)) = listener.accept().await {
        println!("New connection from {:?}", socket.peer_addr().unwrap());
        tokio::spawn(connection::connect_to_server(socket));
    }
}

pub(crate) async fn is_queue_empty() -> bool {
    // this is pub for integration tests
    QUEUE.lock().await.is_empty()
}

pub(crate) mod api_server;
pub(crate) mod connection;
pub(crate) mod messageing;
pub(crate) mod variables;
