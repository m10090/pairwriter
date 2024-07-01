use std::collections::HashMap;

// import alloc
//
use futures::lock::Mutex;
use futures::SinkExt;
use futures::StreamExt;
use tokio::net::TcpStream;
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};

use lazy_static::lazy_static;

type Username = String;
lazy_static! {
    pub static ref QUEUE: Mutex<HashMap<Username, Client>> = Mutex::new(HashMap::new());
}

pub enum Priviledge {
    ReadOnly,
    ReadWrite,
}

pub struct Client {
    pub priviledge: Priviledge,
    pub ws_stream: WebSocketStream<TcpStream>,
}
impl Client {
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
    fn check_message(&self, message: Message) -> Result<Message, String>  {
        match self.priviledge{
            Priviledge::ReadWrite  => Ok(message),
            Priviledge::ReadOnly => {
                Err("You do not have the priviledge to send messages".to_string())
            }
            
        }
    }
    pub async fn read_message(&mut self) -> Result<Message, String> {
        let ws_stream = &mut self.ws_stream;
        let error = Err(format!(
            "Failed to read message\n from client with ip address: {:?}",
            match ws_stream.get_ref().peer_addr() {
                Ok(x) => x.to_string(),
                Err(_) => "unknown".to_string(),
            }
        ));
        match ws_stream.next().await  {
            // and other message types
            Some(Ok(message)) => self.check_message(message),
            _ => error,
        }
    }
}

async fn handle_connection(raw_stream: TcpStream) -> Result<Client, String> {
    match accept_async(raw_stream).await {
        Ok(ws_stream) => Ok(Client {
            priviledge: Priviledge::ReadOnly,
            ws_stream,
        }),
        Err(e) => {
            eprintln!("Error: {:?}", e);
            Err(e.to_string())
        }
    }
}
pub async fn connect_to_server(raw_stream: TcpStream) -> Result<(), String> {
    match handle_connection(raw_stream).await {
        Ok(mut client) => {
            async move {
                client
                    .send_message("Welcome to the server please add your name".into())
                    .await
                    .unwrap_or_else(|err| {
                        eprintln!("{}", err);
                    });
                match client.read_message().await {
                    Ok(message) => {
                        if message.is_text() {
                            let message = message.into_text().unwrap();
                            let mut queue = QUEUE.lock().await;
                            queue.insert(message.to_string(), client);
                        }
                        Ok(())
                    }
                    Err(err) => Err(err),
                }
            }
            .await
        }
        Err(x) => Err(x),
    }
}
