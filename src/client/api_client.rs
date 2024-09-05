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
use tokio_tungstenite::tungstenite::Message;

type Res<T> = io::Result<T>;

#[derive(Debug)]
pub struct ClientApi {
    file_tree: FileTree,
    priviledge: Priviledge,
}

impl ClientApi {
    pub(crate) fn new(
        files: Vec<String>,
        emty_dirs: Vec<String>,
        priviledge: Priviledge,
    ) -> Self {
        Self {
            file_tree: FileTree::build_tree(files, emty_dirs),
            priviledge,
        }
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
                tokio::spawn(client_send_message(msg));
                return Err(io::Error::new(io::ErrorKind::Other, "requesting the file "));
            }
        };
        Ok(file)
    }

    pub async fn read_tx(&mut self, msg: Message) {
        let file_tree = &mut self.file_tree;
        file_tree.handle_msg(msg);
    }

    pub async fn send_rpc(&mut self, rpc: RPC) {
        if self.priviledge == Priviledge::ReadOnly {
            todo!()
        }
        let _ = client_send_message(rpc.encode().unwrap()).await; // this to stop message fluding
    }
    pub async fn edit_buf(&mut self, path: String, pos: Option<usize>, del: Option<isize>, text: &str) {
        if self.priviledge == Priviledge::ReadOnly {
            return;
        }
        let map = &mut self.file_tree.tree;
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

        let rpc = RPC::EditBuffer { path, changes }; // this is safe because this operation is idiempotent
        let _ = client_send_message(rpc.encode().unwrap()).await;
    }
    pub async fn get_file_maps(&mut self) -> (Vec<String>, Vec<String>) {
        self.file_tree.get_maps()
    }
}
