use crate::{
    client::messaging::client_send_message,
    communication::{
        crdt_tree::{client_funcs::PubClientFn as _, FileTree},
        rpc::RPC,
    },
    server::connection::Priviledge,
};
use automerge::{transaction::Transactable, ReadDoc, ROOT};
use std::io;

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

type Res<T> = io::Result<T>;

#[derive(Debug)]
pub struct ClientApi {
    file_tree: FileTree,
    pub priviledge: Priviledge,
    sender: UnboundedSender<RPC>,
    pub receiver: Option<UnboundedReceiver<RPC>>,
}

impl ClientApi {
    pub(crate) fn new(files: Vec<String>, emty_dirs: Vec<String>, priviledge: Priviledge) -> Self {
        let (sender, receiver) = unbounded_channel();
        let receiver = Some(receiver);
        Self {
            file_tree: FileTree::build_tree(files, emty_dirs),
            priviledge,
            sender,
            receiver,
        }
    }

    pub fn get_receiver(&mut self) -> Option<UnboundedReceiver<RPC>> {
        self.receiver.take()
    }

    pub async fn read_file(&mut self, path: String) -> Res<Vec<u8>> {
        let file_tree = &self.file_tree;
        let file = file_tree.read_buf(&path);
        let file = match file {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(e);
            }
            _ => {
                let rpc = RPC::ReqBufferTree { path };
                let msg = rpc.encode().unwrap();
                let _ = client_send_message(msg).await;

                return Err(io::Error::new(io::ErrorKind::Other, "requesting the file"));
            }
        };
        Ok(file)
    }

    pub async fn read_tx(&mut self, rpc: RPC) {
        let file_tree = &mut self.file_tree;
        file_tree.handle_msg(rpc.clone());
        let _ = self.sender.send(rpc);
    }

    pub async fn send_rpc(&mut self, rpc: RPC) {
        if self.priviledge == Priviledge::ReadOnly {
            todo!()
        }
        let _ = client_send_message(rpc.encode().unwrap()).await; // this to stop message fluding
    }
    pub async fn edit_buf(
        &mut self,
        path: String,
        pos: Option<usize>,
        del: Option<isize>,
        text: &str,
    ) {
        if self.priviledge == Priviledge::ReadOnly {
            return;
        }
        let map = &mut self.file_tree.tree;
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

        let rpc = RPC::EditBuffer { path, changes }; // this is safe because this operation is idiempotent
        let _ = client_send_message(rpc.encode().unwrap()).await;
    }
    pub async fn get_file_maps(&self) -> (&Vec<String>, &Vec<String>) {
        self.file_tree.get_maps()
    }
}
