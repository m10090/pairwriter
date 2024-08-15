use tokio::{net::TcpListener, sync::Mutex};
pub mod connection;
use lazy_static::lazy_static;

use crate::communication::crdt_tree::FileTree;

// lazy_static!{
//     static ref CRDT: Mutex<FileTree> = Mutex::new();
// }
pub async fn start_server(port: u16) {
    let url = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&url).await.unwrap(); // panic is needed
                                                           // when there is a connection made to the server
    tokio::spawn(connection::handle_messages());
    while let Ok((socket, _)) = listener.accept().await {
        println!("New connection from {:?}", socket.peer_addr().unwrap());
        tokio::spawn(connection::connect_to_server(socket));
    }
}
