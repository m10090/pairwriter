use automerge::Automerge;
use std::collections::HashMap;

type Filename = String;

#[derive(Debug, Clone)]
pub struct FileTree {
    tree: HashMap<Filename, Automerge>,
    files: Vec<Filename>, // this is not efficient but I don't care
}
#[derive(Debug, Clone)]
pub enum FileErr {
    FileNotFound,
    FileNotOpen,
}


/* impl FileTree {
    pub fn new(mut files: Vec<String>) -> Self {
        let tree = HashMap::new();
        files.sort();
        Self { tree, files }
    }
    /// get the file if found in the tree else return an error [FileErr]
    pub fn open_file(&mut self, filename: String) -> Result<&automerge::Automerge, FileErr> {
        let file = self.tree.get(&filename);
        if file.is_none() && self.files.binary_search(&filename).is_ok() {
            return Err(FileErr::FileNotOpen); // this should make the client ask for the file
        }
        file.ok_or(FileErr::FileNotFound)
    }
    pub fn add_file(&mut self, filename: String) {
        let files = &mut self.files;
        let i = files.binary_search(&filename);
        files.insert(i, filename);
    }
    pub fn add_buf(&mut self, filename: String, buf: Vec<u8>) {
        let buf = Automerge::load(buf);
        self.tree.insert(filename, buf);
    }
    pub fn remove_file(&mut self, filename: String) {
        let files = &mut self.files;
        let i = files.binary_search(&filename);
        files.remove(i);
    }
} */
