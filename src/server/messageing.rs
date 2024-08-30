use futures::future::select_ok;

use super::*;
/// This function will broadcast a message to all connected clients
/// this is public so that it can be used by the server
async fn broadcast_message(msg: Message) -> Result<(), String> {
    dbg!(&msg);
    if is_queue_empty().await {
        use tokio::time::{sleep, Duration};
        sleep(Duration::from_millis(200)).await;
        return Err("No clients connected".to_string());
    }
    let mut queue = QUEUE.lock().await;
    let mut futures = Vec::with_capacity(queue.len());
    for (_, client) in queue.iter_mut() {
        futures.push(client.send_message(msg.clone()));
    }
    let futures = futures::future::join_all(futures).await;
    for i in futures.into_iter() {
        i?
    }
    Ok(())
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
    for (username, client) in queue.iter_mut() {
        futrs.push(Box::pin(async {
            let rpc = client.read_message().await?;
            API.lock()
                .await
                .read_tx(rpc, client, username)
                .await
                .map_err(|_| "error reading the message".to_string())
        }));
    }
    // this will return the frist message it gets
    let res = match select_ok(futrs).await {
        Ok((message, _)) => Ok(message),
        Err(e) => Err(e),
    };

    if res.is_err() {
        // don't forget the free the queue
        drop(queue);
        // could be all clients are closed
        connection::remove_dead_clients().await;
        return Err("this".to_string());
    }
    res
}

pub(crate) async fn handle_messages() -> ! {
    let (tx, mut rx) = mpsc::unbounded_channel();
    TX.set(tx).unwrap();
    loop {
        async {
            println!("waiting for message");
            connection::remove_dead_clients().await;
            let message = tokio::select! {
                client_message = read_message_from_clients() => client_message?,
                Some(server_message) = rx.recv() => server_message ,
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

pub(crate) async fn server_send_message(msg: Message) {
    TX.get().unwrap().send(msg).unwrap(); // todo: remove this panic
}
