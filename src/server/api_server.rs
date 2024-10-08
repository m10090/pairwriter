use std::{env, io};

use super::{connection::Priviledge, CLIENTS_RES, CLIENTS_SEND};
use crate::{
    communication::{
        file_tree::{server_funcs::PubServerFn as _, FileTree},
        rpc::RPC,
    },
    server::messageing::server_send_message,
};

use futures::SinkExt;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug)]
pub struct ServerApi {
    file_tree: FileTree,
    sender: UnboundedSender<RPC>,
    pub receiver: Option<UnboundedReceiver<RPC>>,
}

impl ServerApi {
    pub(crate) fn new_server() -> Self {
        let (sender, receiver) = unbounded_channel();
        Self {
            file_tree: FileTree::build_file_tree(),
            sender,
            receiver: Some(receiver),
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
    pub async fn edit_buf(
        &mut self,
        path: String,
        pos: Option<usize>,
        del: Option<isize>,
        text: &str,
    ) {
        let map = &mut self.file_tree.tree;
        let file = map.get_mut(&path).unwrap();
        let result = file.edit(pos, del, text);
        let rpc = RPC::EditBuffer {
            path,
            changes: result.0,
            old_head_idx: result.1,
            new_heads: result.2,
        };
        server_send_message(rpc.encode().unwrap());
    }

    pub(super) async fn read_rpc(
        &mut self,
        rpc: RPC,
        client: Priviledge,
        username: &str,
    ) -> Result<Message, ()> {
        let file = &mut self.file_tree;
        let result = file.handle_msg(rpc.clone(), Some(client), username).await?; //todo
        let _ = self.sender.send(rpc);
        Ok(result)
    }

    pub fn take_receiver(&mut self) -> UnboundedReceiver<RPC> {
        self.receiver.take().unwrap()
    }

    pub async fn close_connection(&self, username: &str) -> Result<(), String> {
        let mut clients_send = CLIENTS_RES.lock().await;
        match clients_send.remove(username) { 
            // removing the client from CLIENTS_RES will remove it from CLIENTS_SEND in the next iteration
            Some(_c) => {
                Ok(())
            }
            None => Err("Client not found".to_string()),
        }
    }

    pub async fn list_users(&self) -> Vec<String> {
        let clients = CLIENTS_RES.lock().await;
        clients.keys().cloned().collect()
    }

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
            .handle_msg(
                rpc.clone(),
                None,
                &env::var("SERVER_USERNAME").unwrap_or("SERVER".to_string()),
            )
            .await
        {
            server_send_message(x);
        }
    }

    pub async fn get_file_maps(&self) -> (&Vec<String>, &Vec<String>) {
        self.file_tree.get_maps()
    }
}
