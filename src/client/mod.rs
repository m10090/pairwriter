use futures::stream::{SplitSink, SplitStream, StreamExt};
use futures::SinkExt;
use std::sync::OnceLock;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use crate::communication::rpc::RPC;

type WriterWsStream = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

type ReaderWsStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

pub const WRITER_WS_STREAM: OnceLock<Mutex<WriterWsStream>> = OnceLock::new(); // a thread safe one

/// handle the connection to the server and initialize the writer
/// for `client_send_message` function
/// Add the url with **"ws://"** or **"wss://"** prefix
pub async fn connect_as_client(url: String, username: String) {
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect"); // connect to the
                                                                               // server panic is needed here

    let (mut writer, reader) = ws_stream.split();
    let rpc = RPC::AddUsername(username);
    let config = bincode::config::standard();
    let message = Message::binary(bincode::encode_to_vec(rpc, config).unwrap_or(vec![]).as_slice());

    writer
        .send(message)
        .await
        .expect("Failed to send username");

    // set the writer to the global variable
    WRITER_WS_STREAM.set(Mutex::new(writer)).expect(
        "the writer is already initialized to change \n the connection restart the application",
    );

    // handle incoming messages
    tokio::spawn(get_on_message(reader));
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl-c event");
}

include!("./messaging.rs");
