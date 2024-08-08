use std::{
    borrow::Cow,
    fs::{self, File},
    io::{self, Error, Result as Res},
};

use automerge::{transaction::Transactable, ReadDoc, ROOT};

use super::*;

pub mod server_func {
    use super::*;
    pub trait ServerFuncFile {
        fn open_file(&mut self, path: String) -> Res<()>;
        fn create_file(&mut self, path: String) -> Res<()>;
        fn move_file(&mut self, old_path: String, new_path: String) -> Res<()>;
        fn rm_file(&mut self, path: String) -> Res<()>;
    }

    pub trait ServerFuncDir {
        fn move_dir(&mut self, old_path: String, new_path: String) -> Res<()>;
        fn rm_dir(&mut self, path: String) -> Res<()>;
        fn make_dir(&mut self, path: String) -> Res<()>;
    }
    pub trait ServerFuncBuf {
        fn drop_buf(&mut self, path: String) -> Res<()>;
        fn save_buf(self, path: String) -> Res<()>;
    }
}

use server_func::*;

impl ServerFuncFile for FileTree {
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
        }
        Ok(())
    }
}

impl ServerFuncDir for FileTree {
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
            .map(|s| s.replacen(&old_path, &new_path, 1)) // replacen is a must
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

        files.drain(start..end);
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
}

impl ServerFuncBuf for FileTree {
    fn drop_buf(&mut self, path: String) -> Res<()> {
        if self.files.binary_search(&path).is_ok() && self.tree.remove(&path).is_some() {
            Ok(())
        } else {
            Err(Error::new(
                io::ErrorKind::NotConnected,
                "file is not opened or not found",
            ))
        }
    }
    fn save_buf(self, path: String) -> Res<()> {
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
                .unwrap(); // todo: the panic
            if let Ok(text) = file.text(content_exid) {
                fs::write(&path, text)?;
            } else if let Value::Scalar(Cow::Owned(ScalarValue::Bytes(bin))) = bin { // todo: check
                // this
                fs::write(&path, bin)?;
            } else {
                return Err(Error::new(
                    io::ErrorKind::InvalidData,
                    "The file content is not valid",
                ));
            }
        }
        Err(Error::new(
            io::ErrorKind::NotConnected,
            "file is not opened or not found",
        ))
    }
}
