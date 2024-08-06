use super::{assert_ok, assert_vec, expected_err, FILES};
use super::{FileTree, ServerFunc::*};

// to do explane the tests
#[test]
fn remove_dir_1() {
    let mut files = FILES.clone();
    let mut ft = FileTree::new(files.clone(), vec![]);

    assert_ok(ft.rm_dir("./dir1/".to_string()));

    files.retain(|x| !x.starts_with("./dir1/"));

    assert_vec(ft, Some(&files), None);
}

#[test]
fn remove_dir_2() {
    let mut files = FILES.clone();
    let mut ft = FileTree::new(files.clone(), vec![]);

    assert_ok(ft.rm_dir("./dir1/".to_string()));

    files.retain(|x| !x.starts_with("./dir1/"));

    assert_vec(ft, Some(&files), None);
}

#[test]
fn remove_dir_3() {
    let mut files = FILES.clone();
    let mut ft = FileTree::new(files.clone(), vec![]);

    assert_ok(ft.rm_dir("./dir0/".to_string()));
    files.retain(|x| !x.starts_with("./dir0/"));

    assert_vec(ft, Some(&files), None);
}

#[test]
fn remove_dir_4() {
    let mut files = FILES.clone();
    let mut ft = FileTree::new(files.clone(), vec![]);
    assert_ok(ft.rm_dir("./dir0/dir1/".to_string()));
    files.retain(|x| !x.starts_with("./dir0/dir1/"));
    assert_vec(ft, Some(&files), None);
}

#[test]
fn remove_dir_5() {
    let mut files = FILES.clone();
    let mut ft = FileTree::new(files.clone(), vec![]);
    assert_ok(ft.rm_dir("./dir3/".to_string()));

    files.retain(|x| !x.starts_with("./dir3/"));
    assert_vec(ft, Some(&files), None)
}

#[test]
fn remove_dir_6() {
    let files = FILES.clone();
    let mut emty_dirs = vec!["./empty_dir/".to_string()];
    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    assert_ok(ft.rm_dir("./empty_dir/".to_string()));
    emty_dirs.clear();
    assert_vec(ft, Some(&files), Some(&emty_dirs));
}

// todo: add remove_dir error expected_err

#[test]
fn move_dir_1() {
    let mut files = FILES.clone();
    files.push("./dir1./dir1./dir1./file1.txt".to_string());
    let emty_dirs = vec!["./empty_dir/".to_string()];
    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());
    assert_ok(ft.move_dir("./dir1./".to_string(), "./hi/".to_string()));
    files.sort();
    files.remove(
        files
            .binary_search(&"./dir1./dir1./dir1./file1.txt".to_string())
            .unwrap(),
    );
    files.push("./hi/dir1./dir1./file1.txt".to_string());
    files.sort();
    assert_vec(ft, Some(&files), Some(&emty_dirs));
}

#[test]
fn move_dir_2() {
    let mut files = FILES.clone();
    let emty_dirs = vec!["./empty_dir/".to_string()];
    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());
    assert_ok(ft.move_dir("./dir1/".to_string(), "./hi/".to_string()));
    files.retain(|x| !x.starts_with("./dir1/"));
    // "./dir1/file1.txt"
    // "./dir1/file2.txt"
    // "./dir1/subdir/file3.txt"
    let mut files = [
        files,
        vec![
            "./hi/file1.txt".to_string(),
            "./hi/file2.txt".to_string(),
            "./hi/subdir/file3.txt".to_string(),
        ],
    ]
    .concat();
    files.sort();
    assert_vec(ft, Some(&files), Some(&emty_dirs));
}

#[test]
fn make_dir_1() {
    let files = FILES.clone();
    let mut emty_dirs = vec!["./not_empty_dir/emty_dir/".to_string()];

    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());
    assert_ok(ft.make_dir("./not_empty_dir/dir1/".to_string()));
    emty_dirs.push("./not_empty_dir/dir1/".to_string());
    emty_dirs.sort();
    assert_vec(ft, Some(&files), Some(&emty_dirs));
}
#[test]
fn make_dir_2() {
    let files = FILES.clone();
    let mut emty_dirs = vec!["./empty_dir/".to_string()];

    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());
    emty_dirs.clear();
    emty_dirs.push("./empty_dir/empty_dir/".to_string());
    assert_ok(ft.make_dir("./empty_dir/empty_dir/".to_string()));
    assert_vec(ft, Some(&files), Some(&emty_dirs));
}
#[test]
fn move_dir_3() {
    let files = FILES.clone();
    let mut emty_dirs = vec!["./empty_dir/".to_string()];

    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());
    emty_dirs.clear();
    emty_dirs.push("./hi/".to_string());
    assert_ok(ft.move_dir("./empty_dir/".to_string(), "./hi/".to_string()));
    assert_vec(ft, Some(&files), Some(&emty_dirs));
}
#[test]
fn move_dir_4() {
    let files = FILES.clone();
    let emty_dirs = vec!["./empty_dir/".to_string()];

    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    expected_err(ft.make_dir("./dir1/".to_string()));

    assert_vec(ft, Some(&files), Some(&emty_dirs));
}
