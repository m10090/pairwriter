use super::{assert_vec, FILES};
use super::{FileTree, ServerFunc};

// to do explane the tests
#[test]
fn remove_dir() {
    let mut files = FILES.clone();
    let mut ft = FileTree::new(files.clone(), vec![]);

    ft.rm_dir("./dir1/".to_string()).unwrap();

    files.retain(|x| !x.starts_with("./dir1/"));

    assert_vec(ft, Some(&files), None);
}


#[test]
fn remove_emty_dir() {
    let files = FILES.clone();
    let mut emty_dirs = vec!["./empty_dir/".to_string()];
    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    ft.rm_dir("./empty_dir/".to_string()).unwrap();
    emty_dirs.clear();
    assert_vec(ft, Some(&files), Some(&emty_dirs));
}

#[test]
fn remove_dir_resulting_in_empty_dir() {
    let mut files = FILES.clone();
    let mut ft = FileTree::new(files.clone(), vec![]);

    ft.rm_dir("./dir_with_one_dir/dir_with_one_file/".to_string()).unwrap();

    let emty_dirs = vec!["./dir_with_one_dir/".to_string()];

    files.retain(|x| !x.starts_with("./dir_with_one_dir/dir_with_one_file/"));
    files.sort();

    assert_vec(ft, Some(&files), Some(&emty_dirs));
}

#[test]
fn make_dir_expeted_err() {
    let files = FILES.clone();
    let mut emty_dirs = vec!["./empty_dir/".to_string()];
    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());

    ft.make_dir("./dir1/".to_string()).unwrap_err();

    emty_dirs.sort();
    assert_vec(ft, Some(&files), Some(&emty_dirs));
}


#[test]
fn move_dir() {
    let mut files = FILES.clone();
    let emty_dirs = vec!["./empty_dir/".to_string()];
    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());
    ft.move_dir("./dir1/".to_string(), "./hi/".to_string())
        .unwrap();
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
fn make_dir_to_sibling_empty_dir() {
    let files = FILES.clone();
    let mut emty_dirs = vec!["./not_empty_dir/emty_dir/".to_string()];

    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());
    ft.make_dir("./not_empty_dir/dir1/".to_string()).unwrap();
    emty_dirs.push("./not_empty_dir/dir1/".to_string());
    emty_dirs.sort();
    assert_vec(ft, Some(&files), Some(&emty_dirs));
}
#[test]
fn make_dir_inside_an_emty_dir() {
    let files = FILES.clone();
    let mut emty_dirs = vec!["./empty_dir/".to_string()];

    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());
    emty_dirs.clear();
    emty_dirs.push("./empty_dir/empty_dir/".to_string());
    ft.make_dir("./empty_dir/empty_dir/".to_string()).unwrap();
    assert_vec(ft, Some(&files), Some(&emty_dirs));
}
#[test]
fn rename_emty_dir() {
    let files = FILES.clone();
    let mut emty_dirs = vec!["./empty_dir/".to_string()];

    let mut ft = FileTree::new(files.clone(), emty_dirs.clone());
    emty_dirs.clear();
    emty_dirs.push("./hi/".to_string());
    ft.move_dir("./empty_dir/".to_string(), "./hi/".to_string())
        .unwrap();
    assert_vec(ft, Some(&files), Some(&emty_dirs));
}
