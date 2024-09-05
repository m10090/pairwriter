use crate::server::api_server::{self, ServerApi};

use super::*;
use std::sync::OnceLock;
use connection::Client;


lazy_static! {
    pub(super) static ref QUEUE: Mutex<HashMap<String, Client>> = Mutex::new(HashMap::new());
}
lazy_static! {
    pub  static  ref  API: ServerApi = ServerApi::new_server();
}

pub(super) static TX: OnceLock<mpsc::UnboundedSender<Message>> = OnceLock::new();
