use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt as _,
};
use lazy_static::lazy_static;
use std::collections::HashMap;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
// use tokio_tungstenite::tungstenite;

use variables::*;

lazy_static! {
    static ref CURRENT_DIR: String = std::env::current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
        .replace("\\", "/") + "/"; // this will not work in root directory
}
type SinkSend = SplitSink<WebSocketStream<TcpStream>, Message>;
type SinkRes = SplitStream<WebSocketStream<TcpStream>>;

pub async fn start_server(port: u16) {
    {
        use simplelog::{CombinedLogger, Config, LevelFilter, WriteLogger};
        use std::env;
        CombinedLogger::init(vec![WriteLogger::new(
            LevelFilter::Warn,
            Config::default(),
            std::fs::File::create(env::var("LOGFILE").unwrap_or("log.txt".to_string())).unwrap(),
        )])
        .unwrap();
    } // init logger
      // main point
    let url = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&url).await.unwrap(); // panic is needed
                                                           // when there is a connection made to the server
    tokio::spawn(messageing::handle_messages());
    tokio::spawn(watch_file_change());
    while let Ok((socket, _)) = listener.accept().await {
        log::info!("New connection from {:?}", socket.peer_addr().unwrap());
        tokio::spawn(connection::connect_to_server(socket));
    }
}

pub(crate) async fn watch_file_change() {
    use crate::communication::rpc::RPC;
    use notify::DebouncedEvent;
    use notify::{watcher, RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    use std::time::Duration;
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_millis(100)).unwrap();
    watcher.watch(".", RecursiveMode::Recursive).unwrap(); // panic to stop the program
    macro_rules! relative_path {
        ($path:ident, $is_dir:ident) => {{
            let mut relative = $path.to_str().unwrap().replacen(&*CURRENT_DIR, "./", 1);
            if $is_dir {
                relative.push('/');
            }
            relative
        }};
        ($path:ident) => {{
            let  relative = $path
                .to_str()
                .unwrap()
                .replacen(&*CURRENT_DIR, "./", 1);
            relative
        }};
        ($path_old:ident, $path_new:ident, $is_dir:ident) => {{
            let mut relative = $path_old.to_str().unwrap().replacen(&*CURRENT_DIR, "./", 1);
            let mut relative_new = $path_new.to_str().unwrap().replacen(&*CURRENT_DIR, "./", 1);
            if $is_dir {
                relative.push('/');
                relative_new.push('/');
            }
            (relative, relative_new)
        }};
    }

    loop {
        let rpc: RPC;
        match rx.recv() {
            Ok(event) => match event {
                DebouncedEvent::Create(path) => {
                    let is_dir = path.is_dir();
                    // let path = path.to_str().unwrap().to_string();
                    let path = relative_path!(path, is_dir);
                    rpc = match is_dir {
                        true => RPC::CreateDirectory { path },
                        false => RPC::CreateFile { path },
                    }
                }
                DebouncedEvent::Remove(path) => {
                    let path = relative_path!(path);
                    let api = API.lock().await;
                    let is_dir = api
                        .get_file_maps()
                        .await
                        .0
                        .binary_search(&path)
                        .is_err();
                    drop(api);
                    rpc = match is_dir {
                        true => RPC::DeleteDirectory { path },
                        false => RPC::DeleteFile { path },
                    }
                }
                DebouncedEvent::Rename(old_path, new_path) => {
                    let is_dir = new_path.is_dir();
                    let (old_path, new_path) = relative_path!(old_path, new_path, is_dir);
                    match is_dir {
                        true => {
                            rpc = RPC::MoveDirectory {
                                path: old_path,
                                new_path,
                            }
                        }
                        false => {
                            rpc = RPC::MoveFile {
                                path: old_path,
                                new_path,
                            }
                        }
                    }
                }
                _ => {
                    continue;
                }
            },
            Err(e) => {
                log::error!("watch error: {:?}", e);
                continue;
            }
        }
        API.lock().await.send_rpc(rpc).await;
    }
}

pub(crate) async fn no_client_connected() -> bool {
    // this is pub for integration tests
    CLIENTS_RES.lock().await.is_empty()
}

pub(crate) mod api_server;
pub(crate) mod connection;
pub(crate) mod messageing;
#[cfg(test)]
pub(crate) mod test;
pub(crate) mod variables;
