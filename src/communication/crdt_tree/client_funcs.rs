#![allow(private_bounds)]
use crate::communication::rpc::RPC;

use super::*;
use std::io::{self, Error};
use std::path::Path;

type Res<T> = io::Result<T>;

trait PrivateClientFn {
    /// this opens a file and add it to the tree
    /// load the file from the Server
    fn create_file(&mut self, filename: String) -> Res<()>; // EMTY_DIRS_OP
    /// move the file from old path to the new path
    fn move_file(&mut self, old_path: String, new_path: String) -> Res<()>; //EMTY_DIRS_OP
    /// remove the file from the tree
    fn rm_file(&mut self, path: String) -> Res<()>; // EMTY_DIRS_OP
    /// move the directror from old path to the new path
    fn move_dir(&mut self, old_path: String, new_path: String) -> Res<()>; // EMTY_DIRS_OP
    /// remove the directory from the tree
    fn rm_dir(&mut self, path: String) -> Res<()>; // EMTY_DIRS_OP
    /// make a new directory in the tree
    fn make_dir(&mut self, path: String) -> Res<()>; // EMTY_DIRS_OP

    fn edit_buf(&mut self, path: String, changes: &[u8]) -> Res<()>;
}

pub trait PubClientFn: PrivateClientFn {
    fn build_tree(files: Vec<String>, emty_dirs: Vec<String>) -> Self;
    fn handle_msg(&mut self, tx: RPC);
}

impl PrivateClientFn for FileTree {
    /// add a file to FileTree

    fn create_file(&mut self, path: String) -> Res<()> {
        // you should have a message
        let parrent_path = Path::new(&path)
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

        match files.binary_search(&path) {
            Ok(_) => Err(Error::new(
                io::ErrorKind::AlreadyExists,
                "The file already exists",
            )),
            Err(i) => {
                files.insert(i, path);
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
    fn move_dir(&mut self, old_path: String, new_path: String) -> Res<()> {
        if !(Self::valid_dir_path(&new_path) && Self::valid_dir_path(&old_path)) {
            return Err(Error::new(
                io::ErrorKind::InvalidInput,
                "The path should start with './' and end with '/'",
            ));
        }
        if !self.in_dir(&old_path) {
            return Err(Error::new(
                io::ErrorKind::NotFound,
                "The old directory does not exist",
            ));
        }
        if self.in_dir(&new_path) {
            return Err(Error::new(
                io::ErrorKind::AlreadyExists,
                "The new directory does exist",
            ));
        }
        let (files, emty_dirs) = (&mut self.files, &mut self.emty_dirs);
        if let Ok(i) = emty_dirs.binary_search(&old_path) {
            // EMTY_DIRS_OP
            emty_dirs.remove(i);
            match emty_dirs.binary_search(&new_path) {
                Ok(_) => unreachable!(),
                Err(i) => {
                    self.emty_dirs.insert(i, new_path);
                }
            }
            return Ok(());
        }

        let start = files.binary_search(&old_path).unwrap_err();

        // check if the directory is empty
        let mut r = files.len();
        let mut l = start;
        // binary search for the end of the directory
        while l < r {
            let mid = l + (r - l) / 2;
            if files[mid].starts_with(&old_path) {
                l = mid + 1;
            } else {
                r = mid;
            }
        }
        let end = r;

        let new_files: Vec<String> = files
            .drain(start..end)
            .map(|s| {
                self.tree.remove(&s);
                s.replacen(&old_path, &new_path, 1)
            }) // replacen is a must
            // make sure to replace the old_path with the new_path for the frist string
            .collect::<Vec<String>>();

        match files.binary_search(&new_path) {
            Ok(_) => unreachable!(),
            Err(i) => {
                let tail = files.split_off(i);
                files.extend(new_files);
                files.extend(tail);
            }
        };

        Ok(())
    }
    fn rm_dir(&mut self, path: String) -> Res<()> {
        if !self.in_dir(&path) {
            return Err(Error::new(
                io::ErrorKind::NotFound,
                "The directory does not exist",
            ));
        }
        let (files, emty_dirs) = (&mut self.files, &mut self.emty_dirs);
        let parent_dir = Self::parent_dir(&path);

        if let Ok(i) = emty_dirs.binary_search(&path) {
            // EMTY_DIRS_OP
            emty_dirs.remove(i);
            return Ok(());
        }

        let start = files.binary_search(&path).unwrap_err();

        // check if the directory is empty
        let mut r = files.len();
        let mut l = start;
        // binary search for the end of the directory
        while l < r {
            let mid = (l + r) / 2;
            if files[mid].starts_with(&path) {
                l = mid + 1;
            } else {
                r = mid;
            }
        }
        let end = r;

        files.drain(start..end).for_each(|s| {
            self.tree.remove(&s);
            drop(s);
        });

        if !self.in_dir(&parent_dir) {
            match self.emty_dirs.binary_search(&parent_dir) {
                Ok(_) => unreachable!(),
                Err(i) => {
                    self.emty_dirs.insert(i, parent_dir);
                }
            }
        }

        Ok(())
    }
    fn make_dir(&mut self, path: String) -> Res<()> {
        if !Self::valid_dir_path(&path) {
            return Err(Error::new(
                io::ErrorKind::InvalidInput,
                "The path should start with './' and end with '/'",
            ));
        }
        if self.in_dir(&path) {
            return Err(Error::new(
                io::ErrorKind::AlreadyExists,
                "The directory already exists",
            ));
        }
        if let Ok(i) = self.emty_dirs.binary_search_by(|x| {
            if path.starts_with(x) {
                std::cmp::Ordering::Equal
            } else {
                x.cmp(&path)
            }
        }) {
            self.emty_dirs.remove(i);
        }

        // path in an emty_directory
        match self.emty_dirs.binary_search(&path) {
            Ok(_) => unreachable!(),
            Err(i) => {
                self.emty_dirs.insert(i, path.clone());
            }
        }

        Ok(())
    }
    fn edit_buf(&mut self, path: String, changes: &[u8]) -> Res<()> {
        if self.files.binary_search(&path).is_err() {
            return Err(Error::new(io::ErrorKind::NotFound, "File Not Found"));
        }
        if let Some(file) = self.tree.get_mut(&path) {
            file.load_incremental(changes)
                .map_err(Self::err_msg)
                .map_err(|_| Error::new(io::ErrorKind::InvalidData, "Can't merge"))?;
        }
        Ok(()) // here there is no error that is not the case in client
    }
    fn rm_file(&mut self, path: String) -> Res<()> {
        let files = &mut self.files;
        if let Ok(i) = files.binary_search(&path) {
            let path = self.files.remove(i);
            let dir_path = Self::parent_dir(&path);
            if !self.in_dir(&dir_path) {
                // EMTY_DIRS_OP
                match self.emty_dirs.binary_search(&dir_path) {
                    Ok(_) => {
                        unreachable!()
                    }
                    Err(i) => {
                        self.emty_dirs.insert(i, dir_path);
                    }
                }
            }
            self.tree.remove(&path);
            Ok(())
        } else {
            Err(Error::new(
                io::ErrorKind::NotFound,
                "The file does not exist",
            ))
        }
    }
}

impl PubClientFn for FileTree {
    fn build_tree(mut files: Vec<String>, mut emty_dirs: Vec<String>) -> Self {
        files.sort_unstable();
        emty_dirs.sort_unstable();
        FileTree {
            files,
            emty_dirs,
            tree: HashMap::new(),
        }
    }
    fn handle_msg(&mut self, rpc: RPC) {
        match rpc {
            RPC::EditBuffer { path, changes } => {
                self.edit_buf(path, changes.as_ref()).unwrap_or_else(|e| log::error!("{}", e));
            }
            RPC::CreateFile { path } => {
                self.create_file(path)
                    .unwrap_or_else(|e| log::error!("{}", e));
            }
            RPC::MoveFile { path, new_path } => {
                self.move_file(path, new_path)
                    .unwrap_or_else(|e| log::error!("{}", e));
            }
            RPC::DeleteFile { path } => {
                self.rm_file(path).unwrap_or_else(|e| log::error!("{}", e));
            }
            RPC::CreateDirectory { path } => {
                self.make_dir(path).unwrap_or_else(|e| log::error!("{}", e));
            }
            RPC::MoveDirectory { path, new_path } => {
                self.move_dir(path, new_path)
                    .unwrap_or_else(|e| log::error!("{}", e));
            }
            RPC::DeleteDirectory { path } => {
                self.rm_dir(path).unwrap_or_else(|e| log::error!("{}", e));
            }
            RPC::FileSaved { .. } => {
                log::error!("Invalid RPC");
            } // should call the api to remove the dirty
            // bit
            RPC::ResSendFile { path, file } => {
                self.tree
                    .insert(path, automerge::Automerge::load(file.as_slice()).unwrap());
            }
            #[allow(unused_variables)]
            RPC::ResMoveCursor {
                username,
                path,
                position,
            } => todo!(),

            #[allow(unused_variables)]
            RPC::ResMark {
                path,
                s_position,
                e_position,
                username,
            } => {
                todo!() // should call the api of user
            }

            m => log::error!("Invalid RPC message {m:?}"),
        }
    }
}
