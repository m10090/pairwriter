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

    return Err(Error::Io(std::io::Error::new(
        std::io::ErrorKind::NotConnected,
        "WebSocket stream not initialized",
    )));
}
/// create a new on message handler that will handle incoming message
/// takin the consderation the message is received or not
pub(super) fn get_on_message(mut reader: ReaderWsStream) -> impl Future<Output = ()> {
    async move {
        while let Some(message) = reader.next().await {
            let message = message.expect("Failed to get message"); // todo: handle error
            dbg!(&message);

            // #[cfg(feature = "integration_testing_client")]
            tokio::spawn(crate::integration_testing::reseived_message(message));
            
            // todo!("Handle message: {:?}", message);
            // function for testing
        }
    }
}
