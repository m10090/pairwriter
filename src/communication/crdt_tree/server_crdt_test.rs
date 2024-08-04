use super::{server_crdt::*, FileTree};
use ctor::ctor;

#[ctor]
static FILES: Vec<String> = {
    let mut files = [
        "./dir0/dir1/file1.txt",
        "./dir0/dir1/file2.txt",
        "./dir0/dir1/subdir/file3.txt",
        "./dir0/file1.txt",
        "./dir1/file1.txt",
        "./dir1/file2.txt",
        "./dir1/subdir/file3.txt",
        "./dir2/file4.txt",
        "./dir3/file1.txt",
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
fn assert_ok<T, E>(res: Result<T, E>)
where
    E: std::fmt::Debug,
{
    assert!(res.is_ok(), "{:?}", res.err().unwrap());
}
fn expected_err<T, E>(res: Result<T, E>)
where
    T: std::fmt::Debug,
{
    assert!(res.is_err(), "{:?}", res.ok().unwrap());
}

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
