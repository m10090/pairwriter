use std::collections::VecDeque;

use super::crdt_tree::FileTree;
use super::{crdt_tree::server_crdt::ServerTx, rpc::RPC};
use crate::server::connection::Client;
use crate::server::messageing::server_send_message;
use automerge::{transaction::Transactable as _, ReadDoc as _, ROOT};
use tokio::sync::{Mutex, RwLock};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug)]
pub struct ServerApi {
    file_tree: Mutex<FileTree>,
    queue: Mutex<VecDeque<RPC>>,
}

impl ServerApi {
    pub fn new_server() -> Self {
        Self {
            file_tree: Mutex::new(FileTree::build_file_tree()),
            queue: Mutex::new(VecDeque::new()),
        }
    }

    pub async fn read_file_server(&self, path: String) -> std::io::Result<Vec<u8>> {
        let mut file = self.file_tree.lock().await;
        let res_buf = file.read_buf(&path);
        let buf = match res_buf {
            Ok(buf) => buf,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(e);
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotConnected => {
                file.open_file(path.clone());
                file.read_buf(&path)?
            }
            e => {
                return e;
            }
        };
        Ok(buf)
    }

    pub async fn apply_changes(&mut self, path: String, pos: usize, del: isize, text: &str) {
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
        let rpc = RPC::EditBuffer { path, changes };
        server_send_message(rpc.encode().unwrap()).await;
    }

    pub async fn read_tx(
        &mut self,
        rpc: RPC,
        client: &mut Client,
        username: &String,
    ) -> Result<Message, ()> {
        let mut file = self.file_tree.lock().await;
        self.queue.lock().await.push_back(rpc.clone());
        file.handel_msg(rpc, client, username).await //todo
    }
    pub async fn read_rpc(&mut self) -> Option<RPC> {
        let mut queue = self.queue.lock().await;
        queue.pop_front()
    }
    pub async fn get_maps(&self) -> (Vec<String>, Vec<String>) {
        self.file_tree.lock().await.get_maps()
    }
}
