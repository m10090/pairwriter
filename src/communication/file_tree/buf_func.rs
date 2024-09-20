use super::*;
use std::io;
use std::io::Result as Res;
impl FileTree {


    pub(super) fn drop_buf(&mut self, path: String) {
        self.tree.remove(&path);
    }
    pub(crate) fn read_buf(&self, path: &String) -> Res<Vec<u8>> {
        let file = self.tree.get(path);
        if file.is_none() && self.files.binary_search(path).is_ok() {
            Err(Error::new(
                io::ErrorKind::NotConnected,
                "file is not in the memory tree",
            )) // this should make the client ask for the file
        } else {
            let file = file.ok_or(Error::new(
                io::ErrorKind::NotConnected,
                "file not found in the file system tree",
            ))?;
            file.read()
        }

    }
}
