use std::io::Result as Res;
use automerge::{ReadDoc as _, ScalarValue, Value, ROOT};
use std::borrow::Cow;
use std::io;
use super::*;
impl FileTree {
    pub fn add_buf(&mut self, filename: String, buf: &[u8]) -> Res<()> {
        if self.files.binary_search(&filename).is_ok() {
            let buf = automerge::Automerge::load(buf).map_err(|_| {
                Error::new(
                    io::ErrorKind::InvalidData,
                    "The file is not a valid automerge file",
                )
            })?;
            self.tree.insert(filename, buf);
            return Ok(());
        }
        Err(Error::new(
            io::ErrorKind::NotFound,
            "The file does not exist",
        ))
    }
    pub fn drop_buf(&mut self, filename: String) {
        self.tree.remove(&filename);
    }
}
