use super::*;
use automerge::{ReadDoc as _, ScalarValue, Value, ROOT};
use std::io;
use std::io::Result as Res;
impl FileTree {
    pub(super) fn add_buf(&mut self, path: String, buf: &[u8]) -> Res<()> {
        if self.files.binary_search(&path).is_ok() {
            let buf = automerge::Automerge::load(buf).map_err(|_| {
                Error::new(
                    io::ErrorKind::InvalidData,
                    "The file is not a valid automerge file",
                )
            })?;
            self.tree.insert(path, buf);
            return Ok(());
        }
        Err(Error::new(
            io::ErrorKind::NotFound,
            "The file does not exist",
        ))
    }
    pub(super) fn drop_buf(&mut self, path: String) {
        self.tree.remove(&path);
    }
    pub(crate) fn read_buf(&self, path: &String) -> Res<Vec<u8>> {
        let file = self.tree.get(path);
        if file.is_none() && self.files.binary_search(path).is_ok() {
            Err(Error::new(
                io::ErrorKind::NotConnected,
                "file not found in the file",
            )) // this should make the client ask for the file
        } else {
            let file = file.ok_or(Error::new(
                io::ErrorKind::NotConnected,
                "file not found in the file system tree",
            ))?;
            match file.get(ROOT, "content") {
                Ok(content) => {
                    let (val, id) = content.unwrap();
                    if val.is_str() {
                        Ok(file.text(id).unwrap().as_bytes().to_vec())
                    } else {
                        Ok(val.to_bytes().unwrap().to_vec())
                    }
                }
                Err(_) => Err(Error::new(
                    io::ErrorKind::InvalidData,
                    "The file is corrupted",
                )),
            }
        }
    }
}
