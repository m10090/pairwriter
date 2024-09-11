use crate::server::api_server::{self, ServerApi};

use super::*;
use connection::ClientRes;
use futures::stream::SplitSink;
use std::sync::{Arc, OnceLock};
use tokio::net::TcpStream;

lazy_static! {
    pub static ref API: Mutex<ServerApi> = Mutex::new(ServerApi::new_server());
}

lazy_static! {
    pub(super) static ref CLIENTS_RES: Mutex<HashMap<String, Arc<Mutex<ClientRes>>>> =
        Mutex::new(HashMap::new());
}
lazy_static! {
    pub(crate) static ref CLIENTS_SEND: Mutex<HashMap<String, Arc<Mutex<SinkSend>>>> =
        Mutex::new(HashMap::new());
}

pub(super) static TX: OnceLock<mpsc::UnboundedSender<Message>> = OnceLock::new();
