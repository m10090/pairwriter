use std::{
    fs::{self, File},
    io::Write,
    panic,
};

use super::{
    assert_ok, assert_vec, expected_err,
    ServerFunc,
    FileTree, FILES,
};
// TODO: explain the tests
#[test]
fn create_file_1() {
    let mut files = FILES.clone();
    let mut emty_dirs = vec!["./empty_dir/".to_string()];
    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    assert_ok(ft.create_file("./empty_dir/file1.txt".to_string()));

    files.push("./empty_dir/file1.txt".to_string());
    files.sort();

    emty_dirs.clear();

    assert_vec(ft, Some(&files), Some(&emty_dirs));
}

#[test]
fn create_file_2() {
    let files = FILES.clone();
    let emty_dirs = vec!["./empty_dir/".to_string()];
    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    assert_ok(ft.create_file("./empty_dir/file1.txt".to_string()));
}
#[test]
fn create_file_3() {
    let files = vec![];
    let emty_dirs = vec![];
    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());
    expected_err(ft.create_file("./dir/file1.txt".to_string()));
}
#[test]
fn create_file_4() {
    let mut files = vec![];
    let emty_dirs = vec![];
    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());
    assert_ok(ft.create_file("./file.txt".to_string()));

    files.push("./file.txt".to_string());

    assert_vec(ft, Some(&files), Some(&emty_dirs));
}

#[test]
fn create_file_5() {
    let files = FILES.clone();
    let emty_dirs = vec!["./empty_dir/".to_string()];
    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    expected_err(ft.create_file("./dir1/file1.txt".to_string()));

    assert_vec(ft, Some(&files), Some(&emty_dirs));
}
#[test]
fn create_file_6() {
    let mut files = FILES.clone();
    let emty_dirs = vec!["./empty_dir/".to_string()];
    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    assert_ok(ft.create_file("./dir1/new_file.txt".to_string()));

    files.push("./dir1/new_file.txt".to_string());
    files.sort();

    assert_vec(ft, Some(&files), Some(&emty_dirs));
}
#[test]
fn create_file_7() {
    let files = FILES.clone();
    let emty_dirs = vec!["./empty_dir/".to_string()];
    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    expected_err(ft.create_file("./dir1/not_dir/file.txt".to_string()));

    assert_vec(ft, Some(&files), Some(&emty_dirs));
}

#[test]
fn open_file_1() {
    let files = FILES.clone();
    let emty_dirs = vec!["./empty_dir/".to_string()];

    let mut ft = File::create("./file1.txt").unwrap();

    ft.write_all("hello world".as_bytes()).unwrap();

    let _ = panic::catch_unwind(move || {
        let mut fs = FileTree::new(files.clone(), emty_dirs.clone());
        fs.open_file("./file1.txt".to_string()).unwrap();

        assert_vec(fs.clone(), Some(&files), Some(&emty_dirs));

        use automerge::*;

        let auto = fs.tree.get("./file1.txt").unwrap();
        let (v, exid) = auto.get(ROOT, "content").unwrap().unwrap();
        assert!(auto.text(exid).unwrap() == "hello world", "auto: {:?}", v);

        assert_vec(fs.clone(), Some(&files), Some(&emty_dirs));
    });
    fs::remove_file("./file1.txt").unwrap();
}
#[test]
fn move_file_1() {
    let mut files = FILES.clone();
    let emty_dirs = vec![];

    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    assert_ok(ft.move_file(
        "./dir1/file1.txt".to_string(),
        "./dir2/file1.txt".to_string(),
    ));

    files.retain(|x| *x != "./dir1/file1.txt".to_string());
    files.push("./dir2/file1.txt".to_string());
    files.sort();

    assert_vec(ft, Some(&files), Some(&emty_dirs));
}
#[test]
fn move_file_2() {
    let mut files = FILES.clone();
    let mut emty_dirs = vec!["./empty_dir/".to_string()];

    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    assert_ok(ft.move_file(
        "./dir1/file1.txt".to_string(),
        "./empty_dir/file1.txt".to_string(),
    ));

    files.retain(|x| *x != "./dir1/file1.txt".to_string());
    files.push("./empty_dir/file1.txt".to_string());
    files.sort();
    emty_dirs.clear();

    assert_vec(ft, Some(&files), Some(&emty_dirs));
}

#[test]
fn move_file_3() {
    let mut files = FILES.clone();
    let mut emty_dirs = vec![];

    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    assert_ok(ft.move_file(
        "./dir_with_one_file/file.txt".to_string(),
        "./dir1/file3.txt".to_string(),
    ));
    files.retain(|x| *x != "./dir_with_one_file/file.txt".to_string());
    files.push("./dir1/file3.txt".to_string());
    emty_dirs.push("./dir_with_one_file/".to_string());

    emty_dirs.sort();
    files.sort();
    assert_vec(ft, Some(&files), Some(&emty_dirs));
}
#[test]
fn remove_file_1() {
    let mut files = FILES.clone();
    let emty_dirs = vec!["./empty_dir/".to_string()];

    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    assert_ok(ft.rm_file("./dir1/file1.txt".to_string()));

    files.retain(|x| *x != "./dir1/file1.txt".to_string());
    files.sort();

    assert_vec(ft, Some(&files), Some(&emty_dirs));
}

#[test]
fn remove_file_add_emty_dir() {
    let mut files = FILES.clone();
    let mut emty_dirs = vec!["./empty_dir/".to_string()];

    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    assert_ok(ft.rm_file("./dir_with_one_file/file.txt".to_string()));

    files.retain(|x| *x != "./dir_with_one_file/file.txt".to_string());
    files.sort();

    emty_dirs.push("./dir_with_one_file/".to_string());
    emty_dirs.sort();

    assert_vec(ft, Some(&files), Some(&emty_dirs));
}
