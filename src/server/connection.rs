use std::collections::HashMap;
use std::pin::Pin;
use futures::future::select_ok;
use futures::lock::Mutex;
use futures::{Future,  SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use lazy_static::lazy_static;

include!("../communication/rpc.rs");

type Username = String;
lazy_static! {
    pub static ref QUEUE: Mutex<HashMap<Username, Client>> = Mutex::new(HashMap::new());
}

#[derive(Debug)]
pub enum Priviledge {
    ReadOnly = 0,
    ReadWrite = 1,
}
#[derive(Debug)]
pub struct Client {
    pub priviledge: Priviledge,
    pub ws_stream: WebSocketStream<TcpStream>,
}


impl Client {
    // close the connection with the client
    pub fn close(&mut self) {
        // close the connection but doesn't handel freeing memory 
        let _ = self.ws_stream.close(None);
    }
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
    fn check_message(&self, message: Message) -> Result<Message, String> {
        match self.priviledge {
            Priviledge::ReadWrite => Ok(message),
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
        let x = ws_stream.next().await;
        let in_message = if let Some(Ok(message)) = x {
            message
        } else {
            return error;
        };
        self.check_message(in_message)
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
    let mut client = handle_connection(raw_stream).await?;
    client
        .send_message("Welcome to the server please add your name".into())
        .await
        .unwrap_or_else(|err| {
            eprintln!("{}", err);
        });

    let message = client.read_message().await?; 
    if message.is_text() {
        let message = message.into_text().unwrap();
        let mut queue = QUEUE.lock().await;
        if queue.contains_key(&message.to_string()) {
            client
                .send_message("Username already taken".into())
                .await
                .unwrap_or_else(|err| {
                    panic!("{}", err);
                });
            client.close();
            return Err("Username already taken".to_string());
        }
        queue.insert(message, client);
        return Ok(());
    }
    Err("Invalid message".to_string())
}
async fn read_message_from_clients() -> Result<Message, String> {
    let mut queue = QUEUE.lock().await;
    let mut futures: Vec<Pin<Box<dyn Future<Output = Result<Message, String>> + Send + >>> = Vec::new(); 
    for (_, client) in queue.iter_mut() {
        futures.push(Box::pin(client.read_message()));
    }
    let res = match select_ok(futures).await {
        Ok((message,_)) => Ok(message),
        Err(e) => Err(e),
    };
    return res;
}
async fn broadcast_message(message: Message) -> Result<(), String> {
    let mut queue = QUEUE.lock().await;
    let mut futures = Vec::new();
    for (_, client) in queue.iter_mut() {
        futures.push(client.send_message(message.clone()));
    }
    let futures = futures::future::join_all(futures).await;
    for i in futures.into_iter(){
        if let Err(e) = i {
            return Err(e);
        }
    }
    Ok(())
}
pub async fn handle_messages() -> ! {
    loop {
         async {
            let message = read_message_from_clients().await?;
            broadcast_message(message).await?;
            Ok::<(), String>(())
        }.await.unwrap_or_else(|err| {
            eprintln!("{}", err);
        });
        
    }
}
