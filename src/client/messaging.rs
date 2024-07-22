use futures::Future;
/// send a message to the server (if the writer is initialized)
/// else it will return an io error
#[allow(dead_code)]
#[no_mangle]
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
pub fn get_on_message(mut reader: ReaderWsStream) -> impl Future<Output = ()> {
    async move {
        while let Some(message) = reader.next().await {
            let message = message.expect("Failed to get message"); // todo: handle error
            // todo!("Handle message: {:?}", message);
            dbg!(message);
        }
    }
}
