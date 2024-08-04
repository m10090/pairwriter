use automerge::Automerge;
use std::collections::HashMap;
#[derive(Debug, Clone)]
pub struct FileTree {
    tree: HashMap<String, Automerge>,
    files: Vec<String>, // this is not efficient but I don't care
    emty_dirs: Vec<String>, // take care when working with emty_dirs
                        // Every operation on emty_dirs will be commented with EMTY_DIRS_OP
}
pub enum FileErr {
    FileNotFound,
    FileNotOpen,
    FileAlreadyExists,
}

impl FileTree {
    pub fn new(mut files: Vec<String>, mut emty_dirs: Vec<String>) -> Self {
        let tree = HashMap::new();
        files.sort();
        emty_dirs.sort();
        Self {
            tree,
            files,
            emty_dirs,
        }
    }
    /// to work right you need to have the dir_path with ending with '/'
    /// returns `true` of the dir_path is in the tree otherwise `false`
    fn in_dir(&self, dir_path: &String) -> bool {
        if !dir_path.starts_with("./") || !dir_path.ends_with('/') {
            return false;
        }
        if dir_path == "./" {
            return true;
        }
        let (files, emty_dirs) = (&self.files, &self.emty_dirs);
        if files
            .binary_search_by(|x| {
                if x.starts_with(dir_path) {
                    std::cmp::Ordering::Equal
                } else {
                    x.cmp(dir_path)
                }
            })
            .is_ok()
            || emty_dirs.binary_search(dir_path).is_ok()
        {
            return true;
        }
        false
    }
    fn valid_dir_path(dir_path: &str) -> bool {
        dir_path.starts_with("./") && dir_path.ends_with('/')
    }
}
pub mod client_crdt;
pub mod server_crdt;
#[cfg(test)]
pub mod server_crdt_test;
