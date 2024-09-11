use std::collections::VecDeque;
use std::io;

use super::connection::{ClientRes, Priviledge};
use super::{CLIENTS_RES, CLIENTS_SEND};
use crate::communication::crdt_tree::FileTree;
use crate::communication::{crdt_tree::server_funcs::PubServerFn as _, rpc::RPC};
use crate::server::messageing::server_send_message;
use automerge::patches::TextRepresentation;
use automerge::transaction::Transactable;
use automerge::{ReadDoc, ROOT};
use futures::SinkExt;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::{Mutex, Semaphore};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug)]
pub struct ServerApi {
    file_tree: FileTree,
    sender: UnboundedSender<RPC>,
    pub receiver: Mutex<UnboundedReceiver<RPC>>,
}

impl ServerApi {
    pub(crate) fn new_server() -> Self {
        let (sender, receiver) = unbounded_channel();
        Self {
            file_tree: FileTree::build_file_tree(),
            sender: sender,
            receiver: Mutex::new(receiver),
        }
    }

    pub async fn read_file_server(&mut self, path: String) -> io::Result<Vec<u8>> {
        let file = &mut self.file_tree;
        let res_buf = file.read_buf(&path);
        match res_buf {
            Ok(buf) => Ok(buf),
            Err(e) if e.kind() == std::io::ErrorKind::NotConnected => {
                file.open_file(path.clone())?;
                Ok(file.read_buf(&path)?)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(e),
            e => e,
        }
    }

    /// update the buffer in the  path
    /// if del and text is None, then it is an total update operation
    /// if one of them is None, then it does nothing
    /// else it is a splice operation
    pub async fn edit_buf(&mut self, path: String, pos: Option<usize>, del: Option<isize>, text: &str) {
        let map = &mut (&mut self.file_tree).tree;
        let file = map.get_mut(&path).unwrap();
        let obj_id = file.get(ROOT, "content").unwrap().unwrap().1; // to do
        let old_heads = file.get_heads();
        {
            let mut tx = file.transaction();
            if pos.is_none() && del.is_none() {
                let _ = tx.update_text(&obj_id, text);
            } else {
                let _ = tx.splice_text(obj_id, pos.unwrap(), del.unwrap(), text);
            }
            tx.commit();
        }
        let changes = file.save_after(old_heads.as_slice());
        let rpc = RPC::EditBuffer { path, changes };
        server_send_message(rpc.encode().unwrap()).await;
    }

    pub(super) async fn read_rpc(
        &mut self,
        rpc: RPC,
        client: Priviledge,
        username: &String,
    ) -> Result<Message, ()> {
        let file = &mut self.file_tree;
        let result = file.handle_msg(rpc.clone(), Some(client), username).await?; //todo
        let _ = self.sender.send(rpc);
        Ok(result)
    }




    // pub async fn close_connection(&self, username: &String) -> Result<(), String> {
    //     let mut queue = CLIENTS_RES.lock().await;
    //     match queue.remove(username) {
    //         Some(c) => {
    //             Ok(())
    //         }
    //         None => Err("Client not found".to_string()),
    //     }
    // }

    pub async fn change_priviledge(
        &self,
        username: &String,
        priviledge: Priviledge,
    ) -> Result<(), String> {
        let mut queue = CLIENTS_SEND.lock().await;
        if let Some(user) = queue.get_mut(username) {
            CLIENTS_RES
                .lock()
                .await
                .get(username)
                .unwrap()
                .lock()
                .await
                .priviledge = priviledge;
            let rpc = RPC::ChangePriviledge { priviledge };
            let _ = user.lock().await.send(rpc.encode().unwrap()).await;
            Ok(())
        } else {
            Err("Client not found".to_string())
        }
    }

    pub async fn send_rpc(&mut self, rpc: RPC) {
        if let Ok(x) = self
            .file_tree
            .handle_msg(rpc.clone(), None, &"".to_string())
            .await
        {
            server_send_message(x).await;
        }
    }
    pub async fn get_file_maps(&self) -> (&Vec<String>, &Vec<String>) {
        self.file_tree.get_maps()
    }
}
