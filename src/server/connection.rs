use crate::communication;
use tokio::sync::mpsc;
use communication::rpc::RPC;
use futures::future::select_ok;
use futures::lock::Mutex;
use futures::{SinkExt, StreamExt};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::net::TcpStream;
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};

type Username = String;
lazy_static! {
    static ref QUEUE: Mutex<HashMap<Username, Client>> = Mutex::new(HashMap::new());
}
static TX : OnceLock<mpsc::UnboundedSender<Message>> = OnceLock::new();

#[cfg(debug_assertions)]
pub async fn is_queue_empty() -> bool {
    QUEUE.lock().await.is_empty()
}
#[derive(Debug, PartialEq)]
enum Priviledge {
    ReadOnly, // TODO: improve this priviledge
    ReadWrite,
}

#[derive(Debug)]
struct Client {
     priviledge: Priviledge,
     ws_stream: WebSocketStream<TcpStream>,
     open: bool, 
}

impl Client {
    // close the connection with the client
    pub async fn send_message(&mut self, message: Message) -> Result<(), String> {
        let ws_stream = &mut self.ws_stream;
        let error = Err(format!(
            "Failed to send message\n to client with ip address: {:?}",
            ws_stream.get_ref().peer_addr().unwrap()
        ));

        match ws_stream.feed(message.clone()).await {
            Ok(_) => Ok(()),
            _ => error,
        }
    }
    fn check_message(
        &self,
        message: Message,
    ) -> Result<(communication::rpc::RPC, Message), String> {
        let message_vec = message.clone().into_data();
        let config = bincode::config::standard();

        let (rpc, _): (RPC, usize) =
            bincode::decode_from_slice(message_vec.as_slice(), config).unwrap();
        match rpc {
            RPC::DeleteFile { .. }
            | RPC::DeleteDirectory { .. }
            | RPC::MoveFile { .. }
            | RPC::DeleteDirectory { .. }
                if self.priviledge == Priviledge::ReadOnly =>
            {
                Err("you don't have the priviledge to editing file structure".to_string())
            }
            RPC::Error(e) => Err(e),
            RPC::ReadBuffer { .. } | RPC::WriteOnBuffer { .. } | RPC::DeleteOnBuffer { .. } => {
                Err("can't accept_buffer message".to_string())
            }
            _ => Ok((rpc, message)),
        }
    }
    pub async fn read_message(&mut self) -> Result<(RPC, Message), String> {
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
        let in_message = if let Some(Ok(message)) = x {
            message
        } else {
            self.open = false;
            return error;
        };
        self.check_message(in_message)
    }
}


pub async fn server_send_message(msg: Message) {
    TX.get().unwrap().send(msg).unwrap();// todo: remove this panic
}
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
pub async fn connect_to_server(raw_stream: TcpStream) -> Result<(), String> {
    let mut client = handle_connection(raw_stream).await?;
    client
        .send_message("Welcome to the server please add your name".into())
        .await
        .unwrap_or_else(|err| {
            eprintln!("{}", err);
        });
    
    let (rpc, _message /* this is the message in binary format not the rpc */) =
        client.read_message().await.unwrap_or_else(|err| {
            eprintln!("{}", err);
            (RPC::Error(err), Message::binary(vec![0]))
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
        queue.insert(message.into(), client);
        return Ok(());
    }
    Err("Invalid message".to_string())
}
async fn read_message_from_clients() -> Result<Message, String> {
    let mut queue = QUEUE.lock().await;
    let mut futrs = Vec::with_capacity(queue.len());
    if queue.is_empty() {
        drop(queue); // free the lock
        use tokio::time::{sleep, Duration};
        sleep(Duration::from_millis(200)).await;
        return Err("No clients connected".to_string());
    }
    // read message from all clients
    for (_, client) in queue.iter_mut() {
        futrs.push(Box::pin(client.read_message()));
    }
    // this will return the frist message it gets
    let res = match select_ok(futrs).await {
        Ok((message, _)) => Ok(message),
        Err(e) => Err(e),
    };
    dbg!(&res);

    if res.is_err() {
        // don't forget the free the queue
        drop(queue);
        // could be all clients are closed
        remove_dead_clients().await;
    }
    Ok(res?.1) // return the message or an error if there is any
}
/// This function will broadcast a message to all connected clients
/// this is public so that it can be used by the server
async fn broadcast_message(msg: Message) -> Result<(), String> {
    dbg!(&msg);
    let mut queue = QUEUE.lock().await;
    let mut futures = Vec::with_capacity(queue.len());
    if queue.is_empty() {
        drop(queue); // this will free the lock
        use tokio::time::{sleep, Duration};
        sleep(Duration::from_millis(200)).await;
        return Err("No clients connected".to_string());
    }
    for (_, client) in queue.iter_mut() {
        futures.push(client.send_message(msg.clone()));
    }
    let futures = futures::future::join_all(futures).await;
    for i in futures.into_iter() {
        if let Err(e) = i {
            return Err(e);
        }
    }
    Ok(())
}
async fn remove_dead_clients() {
    let mut queue = QUEUE.lock().await;
    queue.retain(|username, client| {
        if !client.open {
            println!("Client with username: {} has disconnected", username);
            return false;
        } 
        true
    });
}
pub async fn handle_messages() -> ! {
    let (tx, mut rx) = mpsc::unbounded_channel();
    TX.set(tx).unwrap(); 
    loop {
        async {
            remove_dead_clients().await;
            let message = tokio::select! {
                client_message = read_message_from_clients() => client_message?,
                Some(server_message) = rx.recv()=> server_message , 
                
            };

            broadcast_message(message).await?;
            Ok::<(), String>(())
        }
        .await
        .unwrap_or_else(|err| {
            eprintln!("{}", err);
        });
    }
}
