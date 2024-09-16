use crate::server::api_server::ServerApi;

use super::*;
use connection::ClientRes;
use std::sync::{Arc, OnceLock};

lazy_static! {
    pub static ref API: Mutex<ServerApi> = Mutex::new(ServerApi::new_server());
    pub(super) static ref CLIENTS_RES: Mutex<HashMap<String, Arc<Mutex<ClientRes>>>> =
        Mutex::new(HashMap::new());
    pub(crate) static ref CLIENTS_SEND: Mutex<HashMap<String, Arc<Mutex<SinkSend>>>> =
        Mutex::new(HashMap::new());
}

pub(super) static TX: OnceLock<mpsc::UnboundedSender<Message>> = OnceLock::new();
