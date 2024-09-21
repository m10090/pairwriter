use super::*;
use futures::future::{BoxFuture, FutureExt};
use serial_test::serial;
use std::error::Error;
use std::io;
use std::thread::sleep;
use std::time::Duration;
use tokio::fs;

#[tokio::test]
#[serial]
async fn watcher_rename_file() {
    let res = async {
        fs::File::create_new("test.txt").await?;
        tokio::spawn(watch_file_change());
        fs::rename("test.txt", "test2.txt").await?;
        
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
    res.unwrap();
}

#[tokio::test]
#[serial]
async fn watcher_rename_dir() {
    let res = async {
        fs::create_dir("emty_dir").await?;
        tokio::spawn(watch_file_change());
        fs::rename("emty_dir", "emty_dir2").await?;

        sleep(Duration::from_secs(2));

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
    res.unwrap();
}
//
#[tokio::test]
#[serial]
async fn watcher_create_file() {
    let res = async {
        tokio::spawn(watch_file_change());
        fs::File::create_new("text.txt").await?;

        let api = API.lock().await;
        let (files, _) = api.get_file_maps().await;
        files
            .binary_search(&"./text.txt".to_string())
            .map_err(|_| io::Error::new(io::ErrorKind::NotFound, ""))?;
        Ok::<_, io::Error>(())
    }
    .await;
    let _ = fs::remove_file("text.txt").await;
    res.unwrap();
}

#[tokio::test]
#[serial]
async fn watcher_create_dir() {
    let res = async {
        tokio::spawn(watch_file_change());
        fs::create_dir("emty_dir").await?;

        let api = API.lock().await;
        let (_, dirs) = api.get_file_maps().await;
        dirs.binary_search(&"./emty_dir/".to_string()).unwrap();
        Ok::<_, io::Error>(())
    }
    .await;
    let _ = fs::remove_dir("emty_dir").await;
    res.unwrap();
}

#[tokio::test]
#[serial]
async fn watcher_delete_file() {
    let res = async {
        fs::File::create_new("text.txt").await.unwrap();
        sleep(Duration::from_secs(2));
        tokio::spawn(watch_file_change());
        sleep(Duration::from_secs(2));
        fs::remove_file("text.txt").await.unwrap();
        sleep(Duration::from_secs(2));

        let api = API.lock().await;
        let (files, _) = api.get_file_maps().await;
        if files.binary_search(&"./text.txt".to_string()).is_ok() {
            return Err(io::Error::new(io::ErrorKind::NotFound, ""));
        };
        Ok::<_, io::Error>(())
    }
    .await;
    let _ = fs::remove_file("text.txt").await;
    res.unwrap();
}

#[tokio::test]
#[serial]
async fn watcher_delete_dir() {
    let res = async {
        fs::create_dir("emty_dir").await?;
        tokio::spawn(watch_file_change());
        fs::remove_dir("emty_dir").await?;

        let api = API.lock().await;
        let (_, dirs) = api.get_file_maps().await;
        if dirs.binary_search(&"./emty_dir/".to_string()).is_ok() {
            return Err(io::Error::new(io::ErrorKind::NotFound, ""));
        };
        Ok::<_, io::Error>(())
    }
    .await;
    let _ = fs::remove_dir("emty_dir").await;
    res.unwrap();
}
