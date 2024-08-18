use std::{
    borrow::Cow,
    fs::{self, File},
    io::{self, Error, Result as Res}, ops::{BitAnd, Not},
};

use crate::server::connection::{Client, Priviledge};
use automerge::{transaction::Transactable, ReadDoc, ScalarValue, Value, ROOT};
use tokio_tungstenite::tungstenite::Message;

use super::*;
use crate::communication::rpc::RPC;

use macros::conditional_pub;

#[conditional_pub(test)]
trait ServerFunc {
    fn open_file(&mut self, path: String) -> Res<()>;
    fn create_file(&mut self, path: String) -> Res<()>;
    fn move_file(&mut self, old_path: String, new_path: String) -> Res<()>;

    fn rm_file(&mut self, path: String) -> Res<()>; // dir operations

    fn move_dir(&mut self, old_path: String, new_path: String) -> Res<()>; // dir operation
    fn rm_dir(&mut self, path: String) -> Res<()>;
    fn make_dir(&mut self, path: String) -> Res<()>;
    // can't find a way to use it
    fn edit_buf(&mut self, path: String, changes: Vec<automerge::Change>) -> Res<()>;

    fn save_buf(&mut self, path: String) -> Res<()>;

    fn read_buf(&mut self, path: String) -> Res<Vec<u8>>;
}

pub trait ServerTx: ServerFunc {
    fn build_file_tree() -> Self;
    async fn handel_messages(
        &mut self,
        tx: RPC,
        client: &mut Client,
        username: &String,
    ) -> Result<Message, ()>;
}

impl ServerFunc for FileTree {
    fn open_file(&mut self, path: String) -> Res<()> {
        if self.files.binary_search(&path).is_err() {
            return Err(Error::new(
                io::ErrorKind::NotFound,
                "The file does not exist",
            ));
        }
        enum FileType {
            Text(String),
            Bin(Vec<u8>),
        }

        let file_content: FileType = match fs::read_to_string(&path) {
            Ok(text) => FileType::Text(text),
            Err(ref e) if e.kind() == io::ErrorKind::InvalidData => FileType::Bin(fs::read(&path)?),
            Err(e) => return Err(e),
        };
        let mut buf = Automerge::new();
        match file_content {
            FileType::Text(file_text) => {
                let mut tx = buf.transaction();
                let i = tx
                    .put_object(ROOT, "content", automerge::ObjType::Text)
                    .unwrap(); // todo: check the error
                tx.splice_text(i, 0, 0, &file_text).unwrap(); // todo: check the error
                tx.commit();
            }
            FileType::Bin(file_bin) => {
                let mut tx = buf.transaction();
                tx.put(ROOT, "content", file_bin).unwrap(); // todo: check the error
                tx.commit();
            }
        }
        self.tree.insert(path, buf);
        Ok(())
    }

    fn create_file(&mut self, path: String) -> Res<()> {
        // check if the directory exists
        let dir_path = Self::parent_dir(&path);
        if !self.in_dir(&dir_path) {
            return Err(Error::new(
                io::ErrorKind::NotFound,
                "The directory does not exist",
            ));
        }
        let (files, emty_dirs) = (&mut self.files, &mut self.emty_dirs);

        let i = files.binary_search(&path);
        if i.is_ok() {
            return Err(Error::new(
                io::ErrorKind::AlreadyExists,
                "The file already exists",
            ));
        }
        let i = i.unwrap_err(); // todo: check the errors

        #[cfg(not(test))]
        File::create(&path)?; // this order is important as faliure in creating the file
                              // would result of the file not being added the tree
        if let Ok(i) = emty_dirs.binary_search(&dir_path) {
            emty_dirs.remove(i); // EMTY_DIRS_OP
        };

        files.insert(i, path);
        dbg!(&files);
        Ok(())
    }

    fn move_file(&mut self, old_path: String, new_path: String) -> Res<()> {
        // you know borrow checker
        let files = &self.files;
        let old_index = match files.binary_search(&old_path) {
            Err(_) => return Err(Error::new(io::ErrorKind::NotFound, "file not found")),
            Ok(old_index) => old_index,
        };

        let new_dir_path = Self::parent_dir(&new_path); // no need to check old path parent
        let old_dir_path = Self::parent_dir(&old_path);
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
            Err(i) => files.insert(i, new_path.clone()),
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
        if let Some(file) = self.tree.remove(&old_path) {
            self.tree.insert(new_path, file);
        };
        Ok(())
    }

    fn rm_file(&mut self, path: String) -> Res<()> {
        let files = &mut self.files;
        if let Ok(i) = files.binary_search(&path) {
            #[cfg(not(test))]
            fs::remove_file(&path)?;
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
        #[cfg(not(test))]
        fs::create_dir_all(&new_path)?; // this will create a new directory if the one doesn't
                                        // this will work in nested case
        #[cfg(not(test))]
        fs::rename(&old_path, &new_path)?;
        // this is awkward
        // help me ")
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

        if let Ok(i) = emty_dirs.binary_search(&path) {
            // EMTY_DIRS_OP
            #[cfg(not(test))]
            fs::remove_dir_all(&path)?;
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

        #[cfg(not(test))]
        fs::remove_dir_all(&path)?;

        files.drain(start..end).for_each(|s| {
            self.tree.remove(&s);
            drop(s);
        });
        Ok(())
    }
    /// should be ending with '/'
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
        #[cfg(not(test))]
        fs::create_dir_all(&path)?;
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

    fn save_buf(&mut self, path: String) -> Res<()> {
        if self.files.binary_search(&path).is_err() {
            return Err(Error::new(
                io::ErrorKind::NotFound,
                "file is not opened or not found",
            ));
        }
        if let Some(file) = self.tree.get(&path) {
            use automerge::{ScalarValue, Value};
            let (bin, content_exid) = file
                .get(ROOT, "content")
                .map_err(|_| {
                    Error::new(io::ErrorKind::InvalidData, "The file content is not valid")
                })?
                .unwrap(); // todo: the error
            if let Ok(text) = file.text(content_exid) {
                fs::write(&path, text)?;
            } else if let Value::Scalar(Cow::Borrowed(ScalarValue::Bytes(bin))) = bin {
                // todo: check this
                fs::write(&path, bin)?;
            } else {
                fs::write(&path, vec![])?;
            }
            Ok(())
        } else {
            Err(Error::new(
                io::ErrorKind::NotConnected,
                "file is not opened or not found",
            ))
        }
    }

    fn edit_buf(&mut self, path: String, changes: Vec<automerge::Change>) -> Res<()> {
        // here error should be sent but in the case of client there shouldn't be any erros
        if self.files.binary_search(&path).is_err() {
            return Err(Error::new(io::ErrorKind::NotFound, "File Not Found"));
        }
        if let Some(file) = self.tree.get_mut(&path) {
            file.apply_changes(changes)
                .map_err(Self::err_msg)
                .map_err(|_| Error::new(io::ErrorKind::InvalidData, "Can't merge"))?;
            Ok(())
        } else {
            Err(Error::new(
                io::ErrorKind::NotConnected,
                "file is not opened or not found",
            ))
        }
    }

    fn read_buf(&mut self, path: String) -> Res<Vec<u8>> {
        if self.files.binary_search(&path).is_err() {
            return Err(Error::new(
                io::ErrorKind::NotFound,
                "file is not opened or not found",
            ));
        }
        if let Some(x) = self.tree.get(&path) {
            Ok(x.save())
        } else {
            self.open_file(path.clone())?;
            self.read_buf(path)
        }
    }
}

impl ServerTx for FileTree {
    fn build_file_tree() -> Self {
        // get all files
        use walkdir::WalkDir;
        let mut files = WalkDir::new("./")
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().display().to_string())
            .collect::<Vec<String>>();
        files.sort();
        let mut res = Self {
            files,
            tree: HashMap::new(),
            emty_dirs: Vec::new(),
        };
        fn is_directory_empty(path: &str) -> Res<bool> {
            let mut entries = fs::read_dir(path)?;
            Ok(entries.next().is_none())
        }
        let mut emty_dirs = WalkDir::new("./")
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
            .map(|e| e.path().display().to_string() + "/") // this result in problems is "./"
            // directory
            .filter(|e| is_directory_empty(e).unwrap_or(false))
            .collect::<Vec<String>>();
        // this is the fix of the "./" problem
        if let Ok(i) = emty_dirs.binary_search(&".//".to_string()) {
            emty_dirs.remove(i);
        }
        if is_directory_empty("./").unwrap() {
            if let Err(i) = emty_dirs.binary_search(&"./".to_string()) {
                emty_dirs.insert(i, "./".to_string());
            }
        }
        res.emty_dirs = emty_dirs;
        res
    }
    async fn handel_messages(
        &mut self,
        tx: RPC,
        client: &mut Client,
        username: &String,
    ) -> Result<Message, ()> {
        match tx {
            RPC::EditBuffer { .. } | RPC::RequestSaveFile { .. }
                if client.priviledge == Priviledge::ReadOnly =>
            {
                eprintln!("Unauthorized access by user {username}");
                eprintln!("user trying to edit file without access {username}");
                Err(())
            }

            RPC::EditBuffer { path, changes } => {
                let changes_arr = changes
                    .clone()
                    .into_iter()
                    .map(|x: Vec<u8>| automerge::Change::from_bytes(x).map_err(Self::err_msg))
                    .collect::<Result<Vec<automerge::Change>, _>>()?;
                self.edit_buf(path.clone(), changes_arr)
                    .map_err(Self::err_msg)?;
                let rpc = RPC::EditBuffer { path, changes };
                Ok(rpc.encode().map_err(Self::err_msg)?)
            }
            RPC::RequestSaveFile { path } => {
                self.save_buf(path.clone()).map_err(Self::err_msg)?;
                let rpc = RPC::ServerFileSaved { path };
                Ok(rpc.encode().map_err(Self::err_msg)?)
            }

            RPC::CreateFile { .. }
            | RPC::CreateDirectory { .. }
            | RPC::DeleteFile { .. }
            | RPC::DeleteDirectory { .. }
            | RPC::MoveFile { .. }
            | RPC::MoveDirectory { .. }
                if client.priviledge == Priviledge::ReadOnly =>
            {
                eprintln!("Unauthorized access by user {username}");
                eprintln!("user trying to edit directory structure without access {username}");
                Err(())
            }

            RPC::CreateFile { path } => {
                self.create_file(path.clone()).map_err(Self::err_msg)?;
                let rpc = RPC::CreateFile { path };
                Ok(rpc.encode().map_err(Self::err_msg)?)
            }

            RPC::CreateDirectory { path } => {
                self.make_dir(path.clone()).map_err(Self::err_msg)?;
                let rpc = RPC::CreateDirectory { path };
                Ok(rpc.encode().map_err(Self::err_msg)?)
            }

            RPC::MoveFile { path, new_path } => {
                self.move_file(path.clone(), new_path.clone())
                    .map_err(Self::err_msg)?;
                let rpc = RPC::MoveFile { path, new_path };
                Ok(rpc.encode().map_err(Self::err_msg)?)
            }

            RPC::MoveDirectory { path, new_path } => {
                self.move_dir(path.clone(), new_path.clone())
                    .map_err(Self::err_msg)?;
                let rpc = RPC::MoveDirectory { path, new_path };
                Ok(rpc.encode().map_err(Self::err_msg)?)
            }

            RPC::DeleteFile { path } => {
                self.rm_file(path.clone()).map_err(Self::err_msg)?;
                let rpc = RPC::DeleteFile { path };
                Ok(rpc.encode().map_err(Self::err_msg)?)
            }

            RPC::DeleteDirectory { path } => {
                self.rm_dir(path.clone()).map_err(Self::err_msg)?;
                let rpc = RPC::DeleteDirectory { path };
                Ok(rpc.encode().map_err(Self::err_msg)?)
            }

            RPC::RequestMark {
                path,
                s_position,
                e_position,
            } => {
                let rpc = RPC::Mark {
                    path,
                    s_position,
                    e_position,
                    username: username.clone(),
                };
                Ok(rpc.encode().map_err(Self::err_msg)?)
            }

            RPC::ClientMoveCursor { path, position } => {
                let rpc = RPC::ResMoveCursor {
                    username: username.clone(),
                    path,
                    position,
                };
                Ok(rpc.encode().map_err(Self::err_msg)?)
            }

            RPC::RequestReadBuffer { path } => {
                let rpc = RPC::ServerSendFile {
                    path: path.clone(),
                    file: self.read_buf(path).map_err(Self::err_msg)?,
                };
                client
                    .send_message(rpc.encode().map_err(Self::err_msg)?)
                    .await
                    .map_err(Self::err_msg)?; // await is needed as I think

                Ok(Message::binary(vec![]))
            }

            e => {
                println!("unhandled message {:?}", e);
                Err(())
            }
        }
    }
}
#[cfg(test)]
mod test {
    use std::panic;

    use super::*;
    #[test]
    fn right_naming() {
        fs::create_dir("./emty_dir/").unwrap();
        let res = panic::catch_unwind(|| {
            let res = FileTree::build_file_tree();
            for i in res.emty_dirs.iter() {
                assert!(FileTree::valid_dir_path(i));
            }
            dbg!(&res.emty_dirs);
            res.emty_dirs
                .binary_search(&"./emty_dir/".to_string())
                .unwrap();
            for i in res.files.iter() {
                assert!(File::open(i).is_ok());
            }
        });
        fs::remove_dir("./emty_dir").unwrap();
        res.unwrap();
    }
}
