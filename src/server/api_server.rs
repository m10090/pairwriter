use std::collections::VecDeque;
use std::io;

use crate::communication::crdt_tree::FileTree;
use crate::communication::{crdt_tree::server_funcs::PubServerFn, rpc::RPC};
use crate::server::connection::Client;
use crate::server::messageing::server_send_message;
use automerge::transaction::Transactable;
use automerge::{ReadDoc as _, ROOT};
use tokio::sync::{Mutex, Semaphore};
use tokio_tungstenite::tungstenite::Message;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender, UnboundedReceiver};
use super::connection::Priviledge;
use super::variables::QUEUE;

#[derive(Debug)]
pub struct ServerApi {
    file_tree: Mutex<FileTree>,
    sender: Mutex<UnboundedSender<RPC>>,
    receiver: Mutex<UnboundedReceiver<RPC>>,
}

impl ServerApi {
    pub(crate) fn new_server() -> Self {
        let (sender, receiver) = unbounded_channel();
        Self {
            file_tree: Mutex::new(FileTree::build_file_tree()),
            sender: Mutex::new(sender),
            receiver: Mutex::new(receiver),
        }
    }

    pub async fn read_file_server(&self, path: String) -> io::Result<Vec<u8>> {
        let mut file = self.file_tree.lock().await;
        let res_buf = file.read_buf(&path);
        let buf = match res_buf {
            Ok(buf) => buf,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(e);
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotConnected => {
                file.open_file(path.clone())?;
                file.read_buf(&path)?
            }
            e => {
                return e;
            }
        };
        Ok(buf)
    }

    /// update the buffer in the  path
    /// if del and text is None, then it is an total update operation
    /// if one of them is None, then it does nothing
    /// else it is a splice operation
    pub async fn edit_buf(
         &self,
        path: String,
        pos: Option<usize>,
        del: Option<isize>,
        text: &str,
    ) {
        let map = &mut self.file_tree.lock().await.tree;
        let file = map.get_mut(&path).unwrap();
        let obj_id = file.get(ROOT, "content").unwrap().unwrap().1; // to do
        {
            let mut tx = file.transaction();
            if pos.is_none() && del.is_none() {
                let _ = tx.update_text(&obj_id, text);
            } else {
                let _ = tx.splice_text(obj_id, pos.unwrap(), del.unwrap(), text);
            }
            tx.commit();
        }
        let change = file.get_last_local_change().unwrap();
        let change_in_bytes = change.raw_bytes().to_vec();
        let changes = vec![change_in_bytes];
        let rpc = RPC::EditBuffer { path, changes };
        server_send_message(rpc.encode().unwrap()).await;
    }

    pub(super) async fn read_rpc(
        &self,
        rpc: RPC,
        client: &mut Client,
        username: &String,
    ) -> Result<Message, ()> {
        let mut file = self.file_tree.lock().await;
        let result = file.handle_msg(rpc.clone(), Some(client), username).await?; //todo
        self.sender.lock().await.send(rpc);
        Ok(result)
    }

    pub async fn queue_pop( &self) -> Option<RPC> {
        self.receiver.lock().await.recv().await
        
    }

    pub async fn get_file_maps(&self) -> (Vec<String>, Vec<String>) {
        self.file_tree.lock().await.get_maps()
    }

    pub async fn close_connection(&self, username: &String) -> Result<(), String> {
        let mut queue = QUEUE.lock().await;
        match queue.remove(username) {
            Some(c) => {
                c.close().await?;
                Ok(())
            }
            None => Err("Client not found".to_string()),
        }
    }

    pub async fn change_priviledge(
        &self,
        username: &String,
        priviledge: Priviledge,
    ) -> Result<(), String> {
        let mut queue = QUEUE.lock().await;
        if let Some(user) = queue.get_mut(username) {
            user.priviledge = priviledge;
            let rpc = RPC::ChangePriviledge { priviledge };
            user.send_message(rpc.encode().unwrap()).await?;
            Ok(())
        } else {
            Err("Client not found".to_string())
        }
    }

    pub async fn send_rpc(&self, rpc: RPC) {
        if let Ok(x) = self
            .file_tree
            .lock()
            .await
            .handle_msg(rpc.clone(), None, &"".to_string())
            .await
        {
            server_send_message(x).await;
        }
    }
}
