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
    pub(super) static ref FILETREE: Mutex<FileTree> = Mutex::new(FileTree::build_file_tree());
}

pub(super) static TX: OnceLock<mpsc::UnboundedSender<Message>> = OnceLock::new();
