use crate::communication::crdt_tree::server_crdt::ServerTx as _;
use crate::communication::rpc::RPC;
use futures::{future::select_ok, TryFutureExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{accept_async, tungstenite::Message};

use super::*;

async fn handle_connection(raw_stream: TcpStream) -> Result<Client, String> {
    match accept_async(raw_stream).await {
        Ok(ws_stream) => Ok(Client {
            priviledge: Priviledge::ReadWrite,
            ws_stream,
            open: true,
        }),
        Err(e) => {
            eprintln!("Error: {:?}", e);
            Err(e.to_string())
        }
    }
}
pub(super) async fn connect_to_server(raw_stream: TcpStream) -> Result<(), String> {
    let mut client = handle_connection(raw_stream).await?;
    client
        .send_message("Welcome to the server please add your name".into())
        .await
        .unwrap_or_else(|err| {
            eprintln!("{}", err);
        });

    let rpc = client.read_message().await.unwrap_or_else(|err| {
        eprintln!("{}", err);
        RPC::Error(err)
    });
    if let RPC::AddUsername(message) = rpc {
        let mut queue = QUEUE.lock().await;
        if queue.contains_key(&message.to_string()) {
            client
                .send_message("Username already taken".into())
                .await
                .unwrap_or_else(|err| {
                    panic!("{}", err);
                });
            // client.close();
            return Err("Username already taken".to_string());
        }
        queue.insert(message, client);
        return Ok(());
    }
    let (files, emty_dirs) = API.lock().await.get_file_maps().await;
    let rpc = RPC::ResConnect {
        username: "Server".to_string(),
        files,
        emty_dirs,
    };
    client.send_message(rpc.encode().unwrap()).await.unwrap_or_else(|err| {
        eprintln!("{}", err);
    });

    Err("Invalid message".to_string())
}
pub(super) async fn remove_dead_clients() {
    let mut queue = QUEUE.lock().await;
    queue.retain(|username, client| {
        if !client.open {
            println!("Client with username: {} has disconnected", username);
            return false;
        }
        true
    });
}

#[derive(Debug, PartialEq)]
pub enum Priviledge {
    ReadOnly, // TODO: improve this priviledge
    ReadWrite,
}

#[derive(Debug)]
pub struct Client {
    pub priviledge: Priviledge,
    ws_stream: WebSocketStream<TcpStream>,
    open: bool,
}

impl Client {
    /// close the connection with the client
    pub async fn send_message(&mut self, message: Message) -> Result<(), String> {
        let ws_stream = &mut self.ws_stream;
        let error = Err(format!(
            "Failed to send message\n to client with ip address: {:?}",
            ws_stream.get_ref().peer_addr().unwrap()
        ));

        match ws_stream.send(message.clone()).await {
            Ok(_) => Ok(()),
            _ => error,
        }
    }
    pub async fn read_message(&mut self) -> Result<RPC, String> {
        let ws_stream = &mut self.ws_stream;
        let error = Err(format!(
            "Failed to read message\n from client with ip address: {:?}",
            match ws_stream.get_ref().peer_addr() {
                Ok(x) => x.to_string(),
                Err(_) => "unknown".to_string(),
            }
        ));
        let x = ws_stream.next().await;
        dbg!(&x);
        let in_message = if let Some(Ok(Message::Binary(message))) = x {
            message
        } else {
            self.open = false;
            return error;
        };
        let rpc = RPC::decode(in_message.as_slice()).map_err(|e| e.to_string())?;
        return Ok(rpc);
    }
}
