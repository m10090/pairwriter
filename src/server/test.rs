use super::*;
use serial_test::serial;
use std::io;
use tokio::fs;
use tokio::time::{sleep, Duration};

lazy_static! {
    static ref RT: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();
}
use ctor::ctor;
#[ctor]
fn start_watch() {
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(watch_file_change());
    });
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(start_server(8080));
    });
    std::thread::sleep(std::time::Duration::from_secs(2));
}

#[test]
#[serial]
fn watcher_rename_file() {
    RT.block_on(async {
        let res = async {
            fs::File::create_new("test.txt").await?;

            fs::rename("test.txt", "test2.txt").await?;
            sleep(Duration::from_secs(5)).await;

            let api = API.lock().await;
            let (files, _) = api.get_file_maps().await;
            files
                .binary_search(&"./test2.txt".to_string())
                .map_err(|_| io::Error::new(io::ErrorKind::NotFound, ""))?;
            drop(api);
            Ok::<_, io::Error>(())
        }
        .await;
        let _ = fs::remove_file("test2.txt").await;
        let _ = fs::remove_file("test.txt").await;
        res
    })
    .unwrap();
}

#[test]
#[serial]
fn watcher_rename_dir() {
    RT.block_on(async {
        let res = async {
            fs::create_dir("emty_dir").await?;

            fs::rename("emty_dir", "emty_dir2").await?;

            sleep(Duration::from_secs(5)).await;

            let api = API.lock().await;
            let (_, emty_dir) = api.get_file_maps().await;
            emty_dir
                .binary_search(&"./emty_dir2/".to_string())
                .map_err(|_| io::Error::new(io::ErrorKind::NotFound, ""))?;
            Ok::<_, io::Error>(())
        }
        .await;
        let _ = fs::remove_dir("emty_dir2").await;
        let _ = fs::remove_dir("emty_dir").await;
        res
    })
    .unwrap();
}
//

#[test]
#[serial]
fn watcher_create_file() {
    RT.block_on(async {
        let res = async {
            sleep(Duration::from_secs(2)).await;
            fs::File::create_new("text.txt").await?;
            sleep(Duration::from_secs(5)).await;
            let api = API.lock().await;
            let (files, _) = api.get_file_maps().await;
            files
                .binary_search(&"./text.txt".to_string())
                .map_err(|_| io::Error::new(io::ErrorKind::NotFound, ""))?;
            Ok::<_, io::Error>(())
        }
        .await;
        let _ = fs::remove_file("text.txt").await;
        res
    })
    .unwrap();
}

#[test]
#[serial]
fn watcher_create_dir() {
    RT.block_on(async {
        let res = async {
            sleep(Duration::from_secs(2)).await;
            fs::create_dir("emty_dir").await?;
            sleep(Duration::from_secs(5)).await;

            let api = API.lock().await;
            let (_, dirs) = api.get_file_maps().await;
            dirs.binary_search(&"./emty_dir/".to_string()).unwrap();
            Ok::<_, io::Error>(())
        }
        .await;
        let _ = fs::remove_dir("emty_dir").await;
        res
    })
    .unwrap();
}

#[test]
#[serial]
fn watcher_delete_file() {
    RT.block_on(async {
        let res = async {
            fs::File::create_new("text.txt").await.unwrap();

            sleep(Duration::from_secs(2)).await;
            fs::remove_file("text.txt").await.unwrap();
            sleep(Duration::from_secs(5)).await;

            let api = API.lock().await;
            let (files, _) = api.get_file_maps().await;
            if files.binary_search(&"./text.txt".to_string()).is_ok() {
                return Err(io::Error::new(io::ErrorKind::NotFound, ""));
            };
            Ok::<_, io::Error>(())
        }
        .await;
        let _ = fs::remove_file("text.txt").await;
        res
    })
    .unwrap();
}

#[test]
#[serial]
fn watcher_delete_dir() {
    RT.block_on(async {
        let res = async {
            fs::create_dir("emty_dir").await?;

            sleep(Duration::from_secs(2)).await;
            fs::remove_dir("emty_dir").await?;
            sleep(Duration::from_secs(3)).await;
            let api = API.lock().await;
            let (_, dirs) = api.get_file_maps().await;
            if dirs.binary_search(&"./emty_dir/".to_string()).is_ok() {
                return Err(io::Error::new(io::ErrorKind::NotFound, ""));
            };
            Ok::<_, io::Error>(())
        }
        .await;
        let _ = fs::remove_dir("emty_dir").await;
        res
    })
    .unwrap();
}
