use std::{
    fs::{self, File},
    io::Write,
    panic,
};

use serial_test::serial;

use super::{assert_vec, FileTree, PrivateServerFn, FILES};
// TODO: explain the tests
#[test]
fn create_file_change_in_emty_dir() {
    let mut files = FILES.clone();
    let mut emty_dirs = vec!["./empty_dir/".to_string()];
    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    ft.create_file("./empty_dir/file1.txt".to_string()).unwrap();

    files.push("./empty_dir/file1.txt".to_string());
    files.sort();

    emty_dirs.clear();

    assert_vec(ft, Some(&files), Some(&emty_dirs));
}

#[test]
fn create_file_without_a_dir() {
    let files = vec![];
    let emty_dirs = vec![];
    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());
    ft.create_file("./dir/file1.txt".to_string()).unwrap_err();
}
#[test]
fn create_file_in_the_main_dir() {
    let mut files = vec![];
    let emty_dirs = vec![];
    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());
    ft.create_file("./file.txt".to_string()).unwrap();

    files.push("./file.txt".to_string());

    assert_vec(ft, Some(&files), Some(&emty_dirs));
}

#[test]
fn create_file_without_a_dir_2() {
    let files = FILES.clone();
    let emty_dirs = vec!["./empty_dir/".to_string()];
    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    ft.create_file("./dir1/file1.txt".to_string()).unwrap_err();

    assert_vec(ft, Some(&files), Some(&emty_dirs));
}
#[test]
fn create_file_without_dir_in_emty_dir() {
    let mut files = FILES.clone();
    let emty_dirs = vec!["./empty_dir/".to_string()];
    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    ft.create_file("./dir1/new_file.txt".to_string()).unwrap();

    files.push("./dir1/new_file.txt".to_string());
    files.sort();

    assert_vec(ft, Some(&files), Some(&emty_dirs));
}
#[test]
fn create_file_without_a_dir_nested_case() {
    let files = FILES.clone();
    let emty_dirs = vec!["./empty_dir/".to_string()];
    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    ft.create_file("./dir1/not_dir/file.txt".to_string())
        .unwrap_err();

    assert_vec(ft, Some(&files), Some(&emty_dirs));
}

#[test]
#[serial]
fn open_file() {
    let files = FILES.clone();
    let emty_dirs = vec!["./empty_dir/".to_string()];

    let mut ft = File::create("./file.txt").unwrap();

    ft.write_all("hello world".as_bytes()).unwrap();

    let res = panic::catch_unwind(move || {
        let mut fs = FileTree::new(files.clone(), emty_dirs.clone());
        fs.open_file("./file.txt".to_string()).unwrap();

        assert_vec(fs.clone(), Some(&files), Some(&emty_dirs));

        let auto = fs.tree.get("./file.txt").unwrap();
        assert!(auto.read().unwrap() == b"hello world");

        assert_vec(fs.clone(), Some(&files), Some(&emty_dirs));
    });
    fs::remove_file("./file.txt").unwrap();
    res.unwrap();
}
#[test]
fn move_file_1() {
    let mut files = FILES.clone();
    let emty_dirs = vec![];

    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    ft.move_file(
        "./dir1/file1.txt".to_string(),
        "./dir2/file1.txt".to_string(),
    )
    .unwrap();

    files.retain(|x| *x != "./dir1/file1.txt".to_string());
    files.push("./dir2/file1.txt".to_string());
    files.sort();

    assert_vec(ft, Some(&files), Some(&emty_dirs));
}
#[test]
fn move_file_to_emty_dir() {
    let mut files = FILES.clone();
    let mut emty_dirs = vec!["./empty_dir/".to_string()];

    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    ft.move_file(
        "./dir1/file1.txt".to_string(),
        "./empty_dir/file1.txt".to_string(),
    )
    .unwrap();

    files.retain(|x| *x != "./dir1/file1.txt".to_string());
    files.push("./empty_dir/file1.txt".to_string());
    files.sort();
    emty_dirs.clear();

    assert_vec(ft, Some(&files), Some(&emty_dirs));
}

#[test]
fn move_file_2() {
    let mut files = FILES.clone();
    let mut emty_dirs = vec![];

    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    ft.move_file(
        "./dir_with_one_file/file.txt".to_string(),
        "./dir1/file3.txt".to_string(),
    )
    .unwrap();
    files.retain(|x| *x != "./dir_with_one_file/file.txt".to_string());
    files.push("./dir1/file3.txt".to_string());
    emty_dirs.push("./dir_with_one_file/".to_string());

    emty_dirs.sort();
    files.sort();
    assert_vec(ft, Some(&files), Some(&emty_dirs));
}

#[test]
fn move_file_failer() {
    let files = FILES.clone();
    let emty_dirs = vec!["./empty_dir/".to_string()];

    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    ft.move_file(
        "./dir0/file1.txt".to_string(),
        "./dir1/file1.txt".to_string(),
    )
    .unwrap_err();

    assert_vec(ft, Some(&files), Some(&emty_dirs));
}

#[test]
fn remove_file() {
    let mut files = FILES.clone();
    let emty_dirs = vec!["./empty_dir/".to_string()];

    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    ft.rm_file("./dir1/file1.txt".to_string()).unwrap();

    files.retain(|x| *x != "./dir1/file1.txt".to_string());
    files.sort();

    assert_vec(ft, Some(&files), Some(&emty_dirs));
}

#[test]
fn remove_file_add_emty_dir() {
    let mut files = FILES.clone();
    let mut emty_dirs = vec!["./empty_dir/".to_string()];

    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    ft.rm_file("./dir_with_one_file/file.txt".to_string())
        .unwrap();

    files.retain(|x| *x != "./dir_with_one_file/file.txt".to_string());
    files.sort();

    emty_dirs.push("./dir_with_one_file/".to_string());
    emty_dirs.sort();

    assert_vec(ft, Some(&files), Some(&emty_dirs));
}

#[test]
#[serial]
fn undo_redo_operation() {
    let files = FILES.clone();
    let emty_dirs = vec!["./empty_dir/".to_string()];

    let mut ft = File::create("./file.txt").unwrap();

    ft.write_all("hello world".as_bytes()).unwrap();

    let res = panic::catch_unwind(move || {
        let mut fs = FileTree::new(files.clone(), emty_dirs.clone());
        fs.open_file("./file.txt".to_string()).unwrap();

        assert_vec(fs.clone(), Some(&files), Some(&emty_dirs));

        let auto = fs.tree.get_mut("./file.txt").unwrap();
        assert!(auto.read().unwrap() == b"hello world");

        auto.edit(None, None, "hello world 2");
        auto.undo();
        assert!(auto.read().unwrap() == b"hello world");
        assert_vec(fs.clone(), Some(&files), Some(&emty_dirs));
    });
    fs::remove_file("./file.txt").unwrap();
    res.unwrap();
}
