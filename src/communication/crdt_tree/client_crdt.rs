use super::*;
use macros::conditional_pub;
use std::io::Result as Res;
use std::io::{self, Error};
use std::path::Path;

#[conditional_pub(test)]
trait ClientFunc {
    /// this opens a file and returns a reference to the file
    fn open_file(&mut self, filename: String) -> Res<&Automerge>;
    /// load the file from the Server
    fn create_file(&mut self, filename: String) -> Res<()>; // EMTY_DIRS_OP
    /// move the file from old path to the new path
    fn move_file(&mut self, old_path: String, new_path: String) -> Res<()>; //EMTY_DIRS_OP
    /// move the directror from old path to the new path
    fn move_dir(&mut self, old_path: String, new_path: String) -> Res<()>; // EMTY_DIRS_OP
    /// remove the directory from the tree
    fn rm_dir(&mut self, path: String) -> Res<()>; // EMTY_DIRS_OP
    /// make a new directory in the tree
    fn make_dir(&mut self, path: String) -> Res<()>; // EMTY_DIRS_OP
    

    fn edit_buf(&mut self, path: String, changes: Vec<automerge::Change>) -> Res<()>;


    fn read_buf(&mut self, path:String) -> Res<Vec<u8>> {
        todo!()
    }
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
    fn edit_buf(&mut self, path: String, changes: Vec<automerge::Change>) -> Res<()> {
        if self.files.binary_search(&path).is_err() {
            return Err(Error::new(
                io::ErrorKind::NotFound,
                "File Not Found",
            ));
        }
        if let Some(file) = self.tree.get_mut(&path) {
            file.apply_changes(changes)
                .map_err(Self::err_msg)
                .map_err(|_| Error::new(io::ErrorKind::InvalidData, "Can't merge"))?;
        }
        Ok(()) // here there is no error that is not the case in server
        
    }
}
