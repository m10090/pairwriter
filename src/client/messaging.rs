
use super::*;
use futures::Future;
use tokio_tungstenite::tungstenite::{Error, Message};

/// send a message to the server (if the writer is initialized)
/// else it will return an io error
pub async fn client_send_message(msg: Message) -> Result<(), Error> {
    // order in messages is not need in most cases
    // (as the tree could handel unordered messages)
    if let Some(writer_mutex) = WRITER_WS_STREAM.get() {
        let mut writer_stream = writer_mutex.lock().await;
        writer_stream.send(msg).await?;
        return Ok(());
    }
    Err(Error::Io(std::io::Error::new(
        std::io::ErrorKind::NotConnected,
        "WebSocket stream not initialized",
    )))
}
/// create a new on message handler that will handle incoming message
/// takin the consderation the message is received or not
#[inline]
#[allow(clippy::manual_async_fn)]
pub(super) fn get_on_message(mut reader: ReaderWsStream) -> impl Future<Output = ()> {
    async move {
        while let Some(message) = reader.next().await {
            let message = message.expect("Failed to get message"); // todo: handle error
            if let Message::Binary(ref message) = message {
                #[cfg(feature = "integration_testing_client")]
                {
                    dbg!(&message);
                    tokio::spawn(crate::integration_testing::reseived_message(
                        Message::binary(message.clone()),
                    ));
                }
                let rpc = RPC::decode(message.as_slice()).expect("Failed to decode message");
                if let RPC::ResConnect {
                    username: _username,
                    files,
                    emty_dirs,
                    priviledge,
                } = rpc
                {
                    API.set(Mutex::new(ClientApi::new(files, emty_dirs, priviledge)))
                        .unwrap();
                } else {
                    match API.get() {
                        Some(api) => api.lock().await.read_tx(rpc).await,
                        None => {

                       
                        },
                    };
                }
            }
        }
    }
}
