use std::sync::Arc;

use crate::communication::rpc::RPC;
use bincode::Decode;
use futures::stream::StreamExt;
use tokio::net::TcpStream;
use tokio_tungstenite::{accept_async, tungstenite::Message};

use super::*;

async fn handle_connection(raw_stream: TcpStream) -> Result<WebSocketStream<TcpStream>, String> {
    match accept_async(raw_stream).await {
        Ok(ws_stream) => Ok(ws_stream),
        Err(e) => {
            eprintln!("Error: {:?}", e);
            Err(e.to_string())
        }
    }
}
pub(super) async fn connect_to_server(raw_stream: TcpStream) -> Result<(), String> {
    let mut ws_stream = handle_connection(raw_stream).await?;
    if let Some(Ok(Message::Binary(rpc))) = ws_stream.next().await {
        if let RPC::AddUsername(username) = RPC::decode(rpc.as_slice()).unwrap() {
            let (files, emty_dirs) = API.get_file_maps().await;
            let rpc = RPC::ResConnect {
                username: "Server".to_string(),
                files,
                emty_dirs,
                priviledge: Priviledge::ReadWrite,
            };
            let _ = ws_stream.send(Message::binary(rpc.encode().unwrap())).await;

            let (send, res) = ws_stream.split();
            let send = Arc::new(Mutex::new(send));
            let client_res = Arc::new(Mutex::new(ClientRes {
                priviledge: Priviledge::ReadWrite,
                resever: res,
                open: true,
            }));
            let mut clients_send = CLIENTS_SEND.lock().await;
            clients_send.insert(username.clone(), send.clone());
            let mut clients_res = CLIENTS_RES.lock().await;
            clients_res.insert(username, client_res.clone());
            // drop(ws_stream);
            // drop(res);
            // drop(send);
            drop(clients_send);
            drop(clients_res);
            return Ok(());
        }
    }
    Err("Invalid message".to_string())
}
pub(crate) async fn remove_dead_clients() {
    let mut clients_res = CLIENTS_RES.lock().await;
    let mut clients_send = CLIENTS_SEND.lock().await;
    for (username, client) in clients_res.iter() {
        if !client.lock().await.open {
            println!("Client with username: {} has disconnected", username);
            clients_send.remove(username);
        }
    }
    clients_res.retain(|username, _client| clients_send.contains_key(username));
}

#[derive(
    Debug, PartialEq, Clone, Copy, bincode::Encode, Decode, serde::Serialize, serde::Deserialize, Eq,
)] // TODO: add privileges to the api
pub enum Priviledge {
    ReadOnly, // TODO: improve this priviledge
    ReadWrite,
}

#[derive(Debug)]
pub(crate) struct ClientRes {
    pub(crate) priviledge: Priviledge,
    resever: SinkRes,
    open: bool,
}

impl ClientRes {
    pub(crate) async fn read_message(&mut self) -> Result<RPC, String> {
        let resever = &mut self.resever;
        let error = Err("Failed to read message\n from client".to_string());
        let x = resever.next().await;
        let in_message = if let Some(Ok(Message::Binary(message))) = x {
            message
        } else {
            self.open = false;
            return error;
        };
        let rpc = RPC::decode(in_message.as_slice()).unwrap();
        Ok(rpc)
    }
}
