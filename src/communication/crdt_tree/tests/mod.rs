
use std::collections::HashMap;

use super::{server_crdt::ServerFunc, FileTree};
use ctor::ctor;


#[ctor]
/** [
        "./file.txt",
        "./dir0/dir1/file1.txt",
        "./dir0/dir1/file2.txt",
        "./dir0/dir1/subdir/file3.txt",
        "./dir0/file1.txt",
        "./dir1/file1.txt",
        "./dir1/file2.txt",
        "./dir1/subdir/file3.txt",
        "./dir2/file4.txt",
        "./dir3/file1.txt",
        "./dir_with_one_file/file.txt"
    ] */
static FILES: Vec<String> = {
    let mut files = [
        "./file.txt",
        "./dir0/dir1/file1.txt",
        "./dir0/dir1/file2.txt",
        "./dir0/dir1/subdir/file3.txt",
        "./dir0/file1.txt",
        "./dir1/file1.txt",
        "./dir1/file2.txt",
        "./dir1/subdir/file3.txt",
        "./dir2/file4.txt",
        "./dir3/file1.txt",
        "./dir_with_one_file/file.txt"
    ]
    .iter()
    .map(|x| x.to_string())
    .collect::<Vec<String>>();
    files.sort();
    files
};
fn is_sorted<T: Ord>(data: &[T]) -> bool {
    data.windows(2).all(|w| w[0] <= w[1])
}
fn contains_all(data: &[String], other: &[String]) -> bool {
    other.iter().all(|x| {
        println!("{}", x);
        data.contains(x)
    })
}
fn assert_vec(ft: FileTree, files: Option<&[String]>, emty_dirs: Option<&[String]>) {
    let emty_dirs = emty_dirs.unwrap_or(&[]);
    let files = files.unwrap_or(&[]);
    let f = |ft_data: &[String], array: &[String]| {
        assert_eq!(
            ft_data.len(),
            array.len(),
            "ft_data: {:?}, array: {:?}",
            ft_data,
            array
        );
        assert!(is_sorted(ft_data));
        assert!(contains_all(ft_data, array));
    };
    f(&ft.files, files);
    f(&ft.emty_dirs, emty_dirs);
}
impl FileTree {
    pub fn new(mut files: Vec<String>, mut emty_dirs: Vec<String>) -> Self {
        let tree = HashMap::new();
        files.sort_unstable();
        emty_dirs.sort_unstable();
        Self {
            tree,
            files,
            emty_dirs,
        }
    }
}

mod server_dir_test; 
mod server_files_test;

