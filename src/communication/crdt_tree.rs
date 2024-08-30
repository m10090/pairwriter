use automerge::transaction::Transactable;
use automerge::Automerge;
use std::collections::HashMap;
use std::io::Error;
use std::path::Path;

#[derive(Debug, Clone)]
pub(crate) struct FileTree {
    pub(crate) tree: HashMap<String, Automerge>,
    files: Vec<String>, // this is not efficient but I don't care
    emty_dirs: Vec<String>, // take care when working with emty_dirs
                        // Every operation on emty_dirs will be commented with EMTY_DIRS_OP
}
impl FileTree {
    /// to work right you need to have the dir_path with ending with '/'
    /// returns `true` of the dir_path is in the tree otherwise `false`
    fn in_dir(&self, dir_path: &String) -> bool {
        if !Self::valid_dir_path(dir_path) {
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
            || emty_dirs
                .binary_search_by(|x| {
                    if x.starts_with(dir_path) {
                        std::cmp::Ordering::Equal
                    } else {
                        x.cmp(dir_path)
                    }
                })
                .is_ok()
        {
            return true;
        }
        false
    }
    #[inline]
    fn valid_dir_path(dir_path: &str) -> bool {
        dir_path.starts_with("./") && dir_path.ends_with('/')
    }
    #[inline]
    fn parent_dir(path: &str) -> String {
        Path::new(&path)
            .parent()
            .unwrap_or(Path::new("./")) // Handle case where there's no parent
            .to_str()
            .unwrap_or("./")
            .to_string()
            + "/"
    }
    fn err_msg(e: impl std::fmt::Display) {
        eprintln!("{}", e);
    }
    /// build the tree from the files and emty_dirs
    /// returns the files and emty_dirs
    pub(crate) fn get_maps(&self) -> (Vec<String>, Vec<String>) {
        (self.files.clone(), self.emty_dirs.clone())
    }
}
pub(crate) mod buf_func;
pub(crate) mod client_funcs;
pub(crate) mod server_funcs;
