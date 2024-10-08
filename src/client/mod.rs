use futures::stream::{SplitSink, SplitStream, StreamExt};
use futures::SinkExt;
use std::sync::OnceLock;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
pub(crate) mod api_client;
pub(crate) mod messaging;

use crate::communication::rpc::RPC;
use api_client::ClientApi;

type WriterWsStream = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

type ReaderWsStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

static WRITER_WS_STREAM: OnceLock<Mutex<WriterWsStream>> = OnceLock::new(); // a thread safe one

pub static API: OnceLock<Mutex<ClientApi>> = OnceLock::new();

/// handle the connection to the server and initialize the writer
/// for `client_send_message` function
/// Add the url with **"ws://"** or **"wss://"** prefix
pub async fn connect_as_client(url: String, username: String) {
    {
        use simplelog::{CombinedLogger, Config, LevelFilter, WriteLogger};
        use std::env;
        CombinedLogger::init(vec![WriteLogger::new(
            LevelFilter::Warn,
            Config::default(),
            std::fs::File::create(env::var("LOGFILE").unwrap_or("log.txt".to_string())).unwrap(),
        )])
        .unwrap();
    } // init logger
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect"); // connect to the
                                                                               // server panic is intented here

    let (mut writer, reader) = ws_stream.split();
    let rpc = RPC::AddUsername(username);
    let message = rpc.encode().unwrap(); // todo handle the error

    writer.send(message).await.expect("Failed to send username");

    let writer_mutex = Mutex::new(writer);
    // set the writer to the global variable
    WRITER_WS_STREAM.set(writer_mutex).expect(
        "the writer is already initialized to change \n the connection restart the application",
    );

    // handle incoming messages
    tokio::spawn(messaging::get_on_message(reader));
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl-c"); 
}
