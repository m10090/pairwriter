use automerge::Automerge;
use std::collections::HashMap;
#[derive(Debug, Clone)]
pub struct FileTree {
    tree: HashMap<String, Automerge>,
    files: Vec<String>, // this is not efficient but I don't care
}
pub enum FileErr {
    FileNotFound,
    FileNotOpen,
    FileAlreadyExists,
}

impl FileTree {
    pub fn new(mut files: Vec<String>) -> Self {
        let tree = HashMap::new();
        files.sort();
        Self { tree, files }
    }
}
pub mod client_crdt;
pub mod server_crdt;
