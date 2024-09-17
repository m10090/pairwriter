use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt as _,
};
use lazy_static::lazy_static;
use notify::{FsEventWatcher, RecommendedWatcher, Watcher};
use std::collections::HashMap;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
// use tokio_tungstenite::tungstenite;

use variables::*;

type SinkSend = SplitSink<WebSocketStream<TcpStream>, Message>;
type SinkRes = SplitStream<WebSocketStream<TcpStream>>;

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




#[cfg(feature = "integration_testing")]
pub async fn no_client_connected() -> bool {
    // this is pub for integration tests
    CLIENTS_RES.lock().await.is_empty()
}

pub(crate) mod api_server;
pub(crate) mod connection;
pub(crate) mod messageing;
pub(crate) mod variables;
#[cfg(test)]
pub(crate) mod test;
