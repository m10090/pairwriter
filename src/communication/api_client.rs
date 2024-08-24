use super::crdt_tree::client_crdt::ClientTx as _;
use super::crdt_tree::FileTree;
use super::rpc::RPC;
use crate::client::messaging::client_send_message;
use crate::server::connection::Priviledge;
use automerge::{transaction::Transactable, ReadDoc, ROOT};
use std::io;
use std::io::Result as Res;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
#[derive(Debug)]
pub struct ClientApi {
    file_tree: Mutex<FileTree>,
    priviledge: Priviledge,
}

impl ClientApi {
    // create a new client
    pub fn new_client(
        files: Vec<String>,
        emty_dirs: Vec<String>,
        priviledge: Priviledge,
    ) -> Self {
        Self {
            file_tree: Mutex::new(FileTree::build_tree(files, emty_dirs)),
            priviledge,
        }
    }

    pub async fn read_file_client(&mut self, path: String) -> Res<Vec<u8>> {
        let file_tree = self.file_tree.lock().await;
        let file = file_tree.read_buf(&path);
        let file = match file {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(e);
            }
            _ => {
                drop(file_tree);
                let rpc = RPC::ReqBufferTree { path };
                let msg = rpc.encode().unwrap();
                tokio::spawn(client_send_message(msg));
                return Err(io::Error::new(io::ErrorKind::Other, "requesting the file "));
            }
        };
        Ok(file)
    }

    pub async fn read_tx(&mut self, msg: Message) {
        let mut file_tree = self.file_tree.lock().await;
        file_tree.handle_msg(msg);
    }

    pub async fn send_rpc(&mut self, rpc: RPC) {
        if self.priviledge == Priviledge::ReadOnly {
            todo!()
        }
        client_send_message(rpc.encode().unwrap()).await; // this to stop message fluding
    }
    pub async fn apply_changes(&mut self, path: String, pos: usize, del: isize, text: &str) {
        if self.priviledge == Priviledge::ReadOnly {
            return;
        }
        let map = &mut self.file_tree.lock().await.tree;
        let file = map.get_mut(&path).unwrap();
        let obj_id = file.get(ROOT, "content").unwrap().unwrap().1; // to do
        {
            let mut tx = file.transaction();
            let _ = tx.splice_text(obj_id, pos, del, text);
            tx.commit();
        }
        let change = file.get_last_local_change().unwrap();
        let change_in_bytes = change.raw_bytes().to_vec();
        let changes = vec![change_in_bytes];
        let rpc = RPC::EditBuffer { path, changes};
        client_send_message(rpc.encode().unwrap()).await;
    }
}
