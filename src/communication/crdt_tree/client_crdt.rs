use super::*;
pub trait ClientFuncFile {
    /// this opens a file and returns a reference to the file
    fn open_file(&mut self, filename: String) -> Result<&Automerge, FileErr>;
    /// this removes the files from the tree if needed
    fn close_file(&mut self, filename: String) -> Result<(), FileErr>;
    /// load the file from the Server
    fn create_file(&mut self, filename: String);
}
pub trait ClientFuncDir {
    fn move_dir(&mut self, old_path: String, new_path: String);
    fn remove_dir(&mut self, path: String);
    fn make_dir(&mut self, path: String);
}
pub trait ClientFuncBuf {
    /// add the buffer (or the automerge tree) to the file tree
    fn add_buf(&mut self, filename: String, buf: Vec<u8>);
    /// drop the buffer from the tree structure
    fn del_buf(&mut self, filename: String) -> Result<(), FileErr>;
    /// move the file from old path to the new path
    fn move_file(&mut self, old_path: String, new_path: String) -> Result<(), FileErr>;
}
impl ClientFuncFile for FileTree {
    /// get the file if found in the tree else return an error [FileErr]
    fn open_file(&mut self, filename: String) -> Result<&automerge::Automerge, FileErr> {
        let file = self.tree.get(&filename);
        if file.is_none() && self.files.binary_search(&filename).is_ok() {
            return Err(FileErr::FileNotOpen); // this should make the client ask for the file
        }
        file.ok_or(FileErr::FileNotFound)
    }

    fn close_file(&mut self, filename: String) -> Result<(), FileErr> {
        if self.tree.remove(&filename).is_none() {
            return Err(FileErr::FileNotFound);
        }
        Ok(())
    }
    fn create_file(&mut self, filename: String) {
        let files = &mut self.files;
        // you should have a message
        let i = files
            .binary_search(&filename)
            .expect_err("file already exists");
        files.insert(i, filename);
    }
}

impl ClientFuncBuf for FileTree {
    fn add_buf(&mut self, filename: String, buf: Vec<u8>) {
        let buf = Automerge::load(buf.as_slice()).expect("invalid buffer");
        self.tree
            .insert(filename, buf)
            .expect("file already loaded");
    }
    fn del_buf(&mut self, filename: String) -> Result<(), FileErr> {
        let files = &mut self.files;
        let i = match files.binary_search(&filename) {
            Err(_) => return Err(FileErr::FileNotFound),
            Ok(i) => i,
        };
        files.remove(i);
        // if the file buffer is found remove it
        self.tree.remove(&filename);
        Ok(())
    }
    fn move_file(&mut self, filename: String, new_filename: String) -> Result<(), FileErr> {
        let files = &mut self.files;
        let old_index = match files.binary_search(&filename) {
            Err(_) => return Err(FileErr::FileNotFound),
            Ok(old_index) => old_index,
        };

        let new_index = match files.binary_search(&new_filename) {
            Ok(_) => return Err(FileErr::FileAlreadyExists),
            Err(i) => i,
        };

        files.remove(old_index);

        files.insert(new_index, new_filename.clone());
        // if the file buffer is found remove it
        self.tree.remove(&filename);
        Ok(())
    }
}

impl ClientFuncDir for FileTree {
    fn move_dir(&mut self, old_path: String, new_path: String) {
        !todo!()
    }
    fn make_dir(&mut self, path: String) {
        !todo!()
    }
    fn remove_dir(&mut self, path: String) {
        !todo!()
    }
}
