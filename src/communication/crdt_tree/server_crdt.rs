use std::{
    fs::{self, File},
    io::{self, Error},
    path::Path,
};

use automerge::{transaction::Transactable, ROOT};

use super::*;

pub mod ServerFunc {
    use super::*;
    pub trait ServerFuncFile {
        fn open_file(&mut self, path: String) -> io::Result<()>;
        fn create_file(&mut self, path: String) -> io::Result<()>;
        fn move_file(&mut self, old_path: String, new_path: String) -> io::Result<()>;
        fn rm_file(&mut self, path: String) -> io::Result<()>;
    }

    pub trait ServerFuncDir {
        fn move_dir(&mut self, old_path: String, new_path: String) -> io::Result<()>;
        fn rm_dir(&mut self, path: String) -> io::Result<()>;
        fn make_dir(&mut self, path: String) -> io::Result<()>;
    }
    pub trait ServerFuncBuf {
        fn add_buf(&mut self, path: String, buf: Vec<u8>) -> io::Result<()>;
        fn close_buffer(&mut self, path: String) -> io::Result<()>;
    }
}
use ServerFunc::*;
impl ServerFuncFile for FileTree {
    fn open_file(&mut self, path: String) -> io::Result<()> {
        let file_text = fs::read_to_string(&path)?; // todo: check the error
        let mut buf = Automerge::new();
        {
            let mut tx = buf.transaction();
            let i = tx
                .put_object(ROOT, "content", automerge::ObjType::Text)
                .unwrap(); // todo: check the error
            tx.splice_text(i, 0, 0, &file_text).unwrap(); // todo: check the error
            tx.commit();
        }
        self.tree.insert(path, buf);
        Ok(())
    }
    fn create_file(&mut self, path: String) -> io::Result<()> {
        // check if the directory exists

        let dir_path = Path::new(&path)
            .parent()
            .unwrap_or(Path::new("./")) // Handle case where there's no parent
            .to_str()
            .unwrap_or("./")
            .to_string()
            + "/";
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

    fn move_file(&mut self, old_path: String, new_path: String) -> io::Result<()> {
        let files = &mut self.files;
        // check if the old_files exists and the new_file does not exist
        if files.binary_search(&old_path).is_ok() && files.binary_search(&new_path).is_err() {
            #[cfg(not(test))]
            fs::rename(&old_path, &new_path)?;
            // remove them with the same order in the files
            // and will not check error because it's
            // checked in the previous if statement

            match files.binary_search(&old_path) {
                Ok(i) => {
                    files.remove(i);
                }
                _ => unreachable!(),
            }
            match files.binary_search(&new_path) {
                Err(j) => {
                    files.insert(j, new_path);
                }
                _ => unreachable!(),
            }
            Ok(())
        } else {
            Err(Error::new(
                io::ErrorKind::NotFound,
                "Either old file does not exist or new file already exists",
            ))
        }
    }
    fn rm_file(&mut self, path: String) -> io::Result<()> {
        let files = &mut self.files;
        if let Ok(i) = files.binary_search(&path) {
            #[cfg(not(test))]
            fs::remove_file(&path)?;
            let path = self.files.remove(i);
            let dir_path = Path::new(&path)
                .parent()
                .unwrap_or(Path::new("./"))
                .to_str()
                .unwrap_or("./")
                .to_string()
                + "/" // needed as the path doesn't contain the '/' at the end
            ;

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
        }
        Ok(())
    }
}

impl ServerFuncDir for FileTree {
    fn move_dir(&mut self, old_path: String, new_path: String) -> io::Result<()> {
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
    fn rm_dir(&mut self, path: String) -> io::Result<()> {
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
    fn make_dir(&mut self, path: String) -> io::Result<()> {
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
        };

        Ok(())
    }
}
