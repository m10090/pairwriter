use std::{
    fs::{self, File},
    io::{self, Error},
};

use automerge::{transaction::Transactable, ROOT};

use super::*;

// TODO: use the FileErr

pub trait ServerFuncFile {
    fn open_file(&mut self, path: String) -> io::Result<()>;
    fn create_file(&mut self, path: String) -> io::Result<()>;
    fn move_file(&mut self, old_path: String, new_path: String) -> io::Result<()>;
    fn delete_file(&mut self, path: String) -> io::Result<()>;
}

pub trait ServerFuncDir {
    fn move_dir(&mut self, old_path: String, new_path: String) -> io::Result<()>;
    fn remove_dir(&mut self, path: String) -> io::Result<()>;
    fn make_dir(&mut self, path: String) -> io::Result<()>;
}

impl ServerFuncFile for FileTree {
    fn open_file(&mut self, path: String) -> io::Result<()> {
        let file_text = fs::read_to_string(&path)?; // todo: check the error
        let mut buf = Automerge::new();
        {
            let mut tx = buf.transaction();
            let i = tx
                .put_object(ROOT, "file", automerge::ObjType::Text)
                .unwrap(); // todo: check the error
            tx.splice_text(i, 0, 0, &file_text).unwrap(); // todo: check the error
        }
        self.tree.insert(path, buf);
        Ok(())
    }
    fn create_file(&mut self, path: String) -> io::Result<()> {
        let files = &mut self.files;
        let i = files.binary_search(&path);
        if i.is_ok() {
            io::Error::new(io::ErrorKind::NotFound, "File not found");
        }
        let i = i.unwrap_err();
        File::create(&path)?;
        files.insert(i, path);
        Ok(())
    }
    fn move_file(&mut self, old_path: String, new_path: String) -> io::Result<()> {
        let files = &mut self.files;
        // check if the old_files exists and the new_file does not exist
        if files.binary_search(&old_path).is_ok() && files.binary_search(&new_path).is_err() {
            fs::rename(&old_path, &new_path)?;
            // remove them with the same order in the files
            // and will not check error because it's
            // checked in the previous if statement
            if let Ok(i) = files.binary_search(&old_path) {
                files.remove(i);
            }
            if let Err(j) = files.binary_search(&new_path) {
                files.insert(j, new_path);
            }
            return Ok(());
        }
        Err(Error::new(
            io::ErrorKind::NotFound,
            "Either old file does not exist or new file already exists",
        ))
    }
    fn delete_file(&mut self, path: String) -> io::Result<()> {
        if let Ok(i) = self.files.binary_search(&path) {
            self.files.remove(i);
            fs::remove_file(&path)?;
        }
        Ok(())
    }
}

impl ServerFuncDir for FileTree {
    fn move_dir(&mut self, old_path: String, new_path: String) -> io::Result<()> {
        let _ = fs::create_dir_all(&new_path); // this will create a new directory if the one doesn't
                                               // this will work in nested case
        fs::rename(&old_path, &new_path)?;
        // this is awkward
        // help me ")
        let new_files_tree = fs::read_dir(".")?
            .map(|x| x.unwrap().path().to_str().unwrap().to_string())
            .collect::<Vec<_>>();
        self.files = new_files_tree;
        Ok(())
    }
    fn remove_dir(&mut self, path: String) -> io::Result<()> {
        let files = &mut self.files;
        let start = files.binary_search(&path).unwrap_err();

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

        files.drain(start..end);
        fs::remove_dir_all(&path)?;
        Ok(())
    }
    fn make_dir(&mut self, path: String) -> io::Result<()> {
        fs::create_dir(&path)?;
        let files = &mut self.files;
        match files.binary_search(&path) {
            Ok(_) => {} // element already in vector @ `pos`
            Err(pos) => files.insert(pos, path),
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::FileTree;
    use super::ServerFuncDir;

    #[test]
    fn remove_dir() {
        {
            // test 1
            let mut files = vec![
                "dir1/file1.txt".to_string(),
                "dir1/file2.txt".to_string(),
                "dir1/subdir/file3.txt".to_string(),
                "dir2/file4.txt".to_string(),
                "file5.txt".to_string(),
            ];
            let mut f = FileTree::new(files.clone());
            let _ = f.remove_dir("dir1".to_string());
            drop(files.drain(0..3));
            assert_eq!(f.files, files);
        }
        {
            // test 2
            let mut files = vec![
                "dir0/dir1/file1.txt".to_string(),
                "dir0/dir1/file2.txt".to_string(),
                "dir0/dir1/subdir/file3.txt".to_string(),
                "dir0/file1.txt".to_string(),
                "dir1/file1.txt".to_string(),
                "dir1/file2.txt".to_string(),
                "dir1/subdir/file3.txt".to_string(),
                "dir2/file4.txt".to_string(),
                "file5.txt".to_string(),
            ];
            let mut f = FileTree::new(files.clone());
            let _ = f.remove_dir("dir1".to_string());
            drop(files.drain(4..7));
            assert_eq!(f.files, files);
        }
        {
            // test3
            let mut files = vec![
                "dir0/dir1/file1.txt".to_string(),
                "dir0/dir1/file2.txt".to_string(),
                "dir0/dir1/subdir/file3.txt".to_string(),
                "dir0/file1.txt".to_string(),
                "dir1/file1.txt".to_string(),
                "dir1/file2.txt".to_string(),
                "dir1/subdir/file3.txt".to_string(),
                "dir2/file4.txt".to_string(),
                "dir3/file1.txt".to_string(),
                "file5.txt".to_string(),
            ];
            let mut f = FileTree::new(files.clone());
            let _ = f.remove_dir("dir0".to_string());
            drop(files.drain(0..4));
            assert_eq!(f.files, files);
        }
        {
            // test4
            let mut files = vec![
                "dir0/dir1/file1.txt".to_string(),
                "dir0/dir1/file2.txt".to_string(),
                "dir0/dir1/subdir/file3.txt".to_string(),
                "dir0/file1.txt".to_string(),
                "dir1/file1.txt".to_string(),
                "dir1/file2.txt".to_string(),
                "dir1/subdir/file3.txt".to_string(),
                "dir2/file4.txt".to_string(),
                "dir3/file1.txt".to_string(),
                "file5.txt".to_string(),
            ];

            let mut f = FileTree::new(files.clone());
            let _ = f.remove_dir("dir0/dir1".to_string());
            drop(files.drain(0..3));
            assert_eq!(f.files, files);
        }
        {
            // test5
            let mut files = vec![
                "dir0/dir1/file1.txt".to_string(),
                "dir0/dir1/file2.txt".to_string(),
                "dir0/dir1/subdir/file3.txt".to_string(),
                "dir0/file1.txt".to_string(),
                "dir1/file1.txt".to_string(),
                "dir1/file2.txt".to_string(),
                "dir1/subdir/file3.txt".to_string(),
                "dir2/file4.txt".to_string(),
                "dir3/file1.txt".to_string(),
            ];
            let mut f = FileTree::new(files.clone());
            let _ = f.remove_dir("dir3".to_string());
            drop(files.drain(8..=8));
            assert_eq!(f.files, files);
        }
    }
}
