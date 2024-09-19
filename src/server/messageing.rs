use futures::future::select_ok;

use super::*;

pub(crate) const RESET_WAITING: Message = Message::Binary(vec![]);

/// This function will broadcast a message to all connected clients
/// this is public so that it can be used by the server
async fn broadcast_message(msg: Message) -> Result<(), String> {
    if no_client_connected().await {
        use tokio::time::{sleep, Duration};
        sleep(Duration::from_millis(1000)).await;
        return Err("No clients connected".to_string());
    }
    let clients_send = CLIENTS_SEND.lock().await;
    let mut futures = Vec::with_capacity(clients_send.len());
    for (_, client) in clients_send.iter() {
        let client = client.clone();
        let msg = msg.clone();
        futures.push(async move { client.lock().await.send(msg).await });
    }
    // drop(clients_send);// freeing the client send
    let futures = futures::future::join_all(futures).await;
    for i in futures.into_iter() {
        i.map_err(|e| e.to_string())?
    }
    Ok(())
}
async fn handle_message() -> Result<Message, String> {
    let client_res = CLIENTS_RES.lock().await;
    let mut futrs = Vec::with_capacity(client_res.len());
    if client_res.is_empty() {
        drop(client_res); // free the lock

        use tokio::time::{sleep, Duration};
        sleep(Duration::from_millis(200)).await;

        return Err("No clients connected".to_string());
    }
    // read message from all clients
    for (username, client) in client_res.iter() {
        let client = client.clone();
        let username = username.clone();
        futrs.push(Box::pin(async move {
            let priviledge = client.lock().await.priviledge;
            let rpc = client.lock().await.read_message().await?;
            API.lock()
                .await
                .read_rpc(rpc, priviledge, &username)
                .await
                .map_err(|_| "error reading the message".to_string())
        }));
    }
    drop(client_res); //free the lock to
                      // this will return the first message it gets
    let res = match select_ok(futrs).await {
        Ok((message, _)) => Ok(message),
        Err(e) => Err(e),
    };

    if res.is_err() {
        // could be all clients are closed
        connection::remove_dead_clients().await;
        return Err("all client send error".to_string());
    }
    res
}

pub(crate) async fn handle_messages() -> ! {
    let (tx, mut rx) = mpsc::unbounded_channel();
    TX.set(tx.clone()).unwrap();
    loop {
        async {
            connection::remove_dead_clients().await;
            let message = tokio::select! {
                client_message = handle_message() => client_message?,
                Some(server_message) = rx.recv() => server_message ,
            };
            if message.is_empty() {
                return Ok(());
            }
            broadcast_message(message).await?;
            Ok::<(), String>(())
        }
        .await
        .unwrap_or_else(|err| {
            log::error!("{}", err);
        });
    }
}

pub(crate) fn server_send_message(msg: Message) {
    TX.get().unwrap().send(msg).unwrap(); // todo: remove this panic
}
