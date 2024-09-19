use std::collections::HashMap;

use super::*;
use ctor::ctor;
use serial_test::serial;
use std::panic;

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
    "./dir_with_one_dir/dir_with_one_file/file.txt",
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
        "./dir_with_one_file/file.txt",
        "./dir_with_one_dir/dir_with_one_file/file.txt",
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
        log::info!("{}", x);
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

#[test]
#[serial]
fn right_naming() {
    fs::create_dir("./emty_dir/").unwrap();
    let res = panic::catch_unwind(|| {
        let res = FileTree::build_file_tree();
        for i in res.emty_dirs.iter() {
            assert!(FileTree::valid_dir_path(i));
        }
        res.emty_dirs
            .binary_search(&"./emty_dir/".to_string())
            .unwrap();
        for i in res.files.iter() {
            assert!(File::open(i).is_ok());
        }
    });
    fs::remove_dir("./emty_dir/").unwrap();
    res.unwrap();
}

mod server_dir_test;
mod server_files_test;
