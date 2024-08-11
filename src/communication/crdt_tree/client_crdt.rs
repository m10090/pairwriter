use super::*;
use std::io::Result as Res;
use std::io::{self, Error};
use std::path::Path;

pub trait ClientFunc {
    /// this opens a file and returns a reference to the file
    fn open_file(&mut self, filename: String) -> Res<&Automerge>;
    /// load the file from the Server
    fn create_file(&mut self, filename: String) -> Res<()>;
    /// move the file from old path to the new path
    fn move_file(&mut self, old_path: String, new_path: String) -> Res<()>;
    /// move the directror from old path to the new path
    fn move_dir(&mut self, old_path: String, new_path: String) -> Res<()>;
    /// remove the directory from the tree
    fn rm_dir(&mut self, path: String) -> Res<()>;
    /// make a new directory in the tree
    fn make_dir(&mut self, path: String) -> Res<()>;
    /// add the buffer (or the automerge tree) to the file tree
    fn add_buf(&mut self, filename: String, buf: &[u8]) -> Res<()>;
    /// drop the buffer from the tree structure
    fn drop_buf(&mut self, filename: String);
}
impl ClientFunc for FileTree {
    /// get the file if found in the tree else return an error [FileErr]
    fn open_file(&mut self, filename: String) -> Res<&automerge::Automerge> {
        let file = self.tree.get(&filename);
        if file.is_none() && self.files.binary_search(&filename).is_ok() {
            Err(Error::new(
                io::ErrorKind::NotConnected,
                "file not found in the file",
            )) // this should make the client ask for the file
        } else {
            file.ok_or(Error::new(
                io::ErrorKind::NotConnected,
                "file not found in the file system tree",
            ))
        }
    }

    fn create_file(&mut self, filename: String) -> Res<()> {
        // you should have a message
        let parrent_path = Path::new(&filename)
            .parent()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            + "/";

        if !self.in_dir(&parrent_path) {
            return Err(Error::new(
                io::ErrorKind::NotFound,
                "The directory does not exist",
            ));
        }

        let (files, emty_dir) = (&mut self.files, &mut self.emty_dirs);
        match emty_dir.binary_search(&parrent_path) {
            Ok(_) => (),
            Err(i) => emty_dir.insert(i, parrent_path),
        }

        match files.binary_search(&filename) {
            Ok(_) => Err(Error::new(
                io::ErrorKind::AlreadyExists,
                "The file already exists",
            )),
            Err(i) => {
                files.insert(i, filename);
                Ok(())
            }
        }
    }
    fn move_file(&mut self, old_path: String, new_path: String) -> Res<()> {
        // you know borrow checker
        let files = &self.files;
        let old_index = match files.binary_search(&old_path) {
            Err(_) => return Err(Error::new(io::ErrorKind::NotFound, "file not found")),
            Ok(old_index) => old_index,
        };

        let new_dir_path = Path::new(&new_path)
            .parent()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            + "/"; // no need to check old path parent
        let old_dir_path = Path::new(&old_path)
            .parent()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            + "/";
        if !self.in_dir(&new_dir_path) {
            return Err(Error::new(
                io::ErrorKind::NotFound,
                "The directory does not exist",
            ));
        }
        let files = &mut self.files;

        files.remove(old_index);
        match files.binary_search(&new_path) {
            Ok(_) => {
                return Err(Error::new(
                    io::ErrorKind::AlreadyExists,
                    "file path already exists",
                ))
            }
            Err(i) => files.insert(i, new_path),
        }

        match self.emty_dirs.binary_search(&new_dir_path) {
            Err(_) => (),
            Ok(i) => {
                self.emty_dirs.remove(i);
            }
        }
        if !self.in_dir(&old_dir_path) {
            let emty_dirs = &mut self.emty_dirs;
            match emty_dirs.binary_search(&old_dir_path) {
                Ok(_) => unreachable!(),
                Err(i) => emty_dirs.insert(i, old_dir_path),
            }
        }

        self.tree.remove(&old_path);
        Ok(())
    }
    fn add_buf(&mut self, filename: String, buf: &[u8]) -> Res<()> {
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
    fn drop_buf(&mut self, filename: String) {
        self.tree.remove(&filename);
    }
    fn move_dir(&mut self, old_path: String, new_path: String) -> Res<()> {
        todo!()
    }
    fn make_dir(&mut self, path: String) -> Res<()> {
        todo!()
    }
    fn rm_dir(&mut self, path: String) -> Res<()> {
        todo!()
    }
}
