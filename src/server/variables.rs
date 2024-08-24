use crate::communication::api_server::{self, ApiServer};

use super::*;
use std::sync::OnceLock;
use connection::Client;

lazy_static! {
    pub(super) static ref CRDT: Mutex<FileTree> = Mutex::new(FileTree::build_file_tree());
}

lazy_static! {
    pub(super) static ref QUEUE: Mutex<HashMap<String, Client>> = Mutex::new(HashMap::new());
}
lazy_static! {
    pub  static  ref  API: Mutex<ApiServer> = Mutex::new(ApiServer::new_server());
}

pub(super) static TX: OnceLock<mpsc::UnboundedSender<Message>> = OnceLock::new();
