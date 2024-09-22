use super::*;
pub(crate) async fn watch_file_change() {
    // yes this is writen by chatgpt
    use crate::communication::rpc::RPC;
    use notify::DebouncedEvent;
    use notify::{watcher, RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    // Create a channel to receive file system events
    let (tx, rx) = channel();
    
    // Create a file watcher with a debounce time of 100 milliseconds
    let mut watcher = watcher(tx, Duration::from_millis(100)).unwrap();
    
    // Watch the current directory (.) recursively, meaning all subdirectories and files are watched
    watcher.watch(".", RecursiveMode::Recursive).unwrap(); // Panic if watcher fails to initialize

    loop {
        let rpc: RPC;

        // Wait for the next file system event
        match rx.recv() {
            Ok(event) => match event {
                // Handle file or directory creation events
                DebouncedEvent::Create(path) => {
                    let is_dir = path.is_dir();  // Check if the path is a directory
                    
                    // Manually expand the relative_path macro for this case
                    let mut relative = path.to_str().unwrap().replacen(&*CURRENT_DIR, "./", 1);
                    if is_dir {
                        relative.push('/');
                    }
                    let path = relative;

                    // Create the appropriate RPC message based on whether the path is a directory or file
                    rpc = if is_dir {
                        RPC::CreateDirectory { path }  // CreateDirectory RPC if it's a directory
                    } else {
                        RPC::CreateFile { path }  // CreateFile RPC if it's a file
                    }
                }
                
                // Handle file or directory removal events
                DebouncedEvent::Remove(path) => {
                    // Manually expand the relative_path macro for this case
                    let relative = path.to_str().unwrap().replacen(&*CURRENT_DIR, "./", 1);
                    let mut path = relative;

                    // Lock the API and check if the path is a directory or a file
                    let api = API.lock().await;
                    let is_dir = api
                        .get_file_maps()
                        .await
                        .0
                        .binary_search(&path)
                        .is_err();  // If the file doesn't exist in the map, assume it's a directory
                    
                    // If it's a directory, ensure it has a trailing slash
                    if is_dir {
                        path.push('/');
                    }

                    // Drop the API lock after use
                    drop(api);

                    // Create the appropriate RPC message based on whether the path is a directory or file
                    rpc = if is_dir {
                        RPC::DeleteDirectory { path }  // DeleteDirectory RPC if it's a directory
                    } else {
                        RPC::DeleteFile { path }  // DeleteFile RPC if it's a file
                    }
                }

                // Handle file or directory rename events
                DebouncedEvent::Rename(old_path, new_path) => {
                    let is_dir = new_path.is_dir();  // Check if the new path is a directory
                    
                    // Manually expand the relative_path macro for this case
                    let mut relative_old = old_path.to_str().unwrap().replacen(&*CURRENT_DIR, "./", 1);
                    let mut relative_new = new_path.to_str().unwrap().replacen(&*CURRENT_DIR, "./", 1);
                    if is_dir {
                        relative_old.push('/');
                        relative_new.push('/');
                    }
                    let (old_path, new_path) = (relative_old, relative_new);

                    // Create the appropriate RPC message based on whether the paths refer to a directory or file
                    rpc = if is_dir {
                        RPC::MoveDirectory {
                            path: old_path,
                            new_path,
                        }  // MoveDirectory RPC if it's a directory
                    } else {
                        RPC::MoveFile {
                            path: old_path,
                            new_path,
                        }  // MoveFile RPC if it's a file
                    }
                }
                
                // Ignore other event types (e.g., modifications, access)
                _ => {
                    continue;
                }
            },

            // Handle errors from the watcher
            Err(e) => {
                log::error!("watch error: {:?}", e);
                continue;
            }
        }

        // Send the constructed RPC to the API asynchronously
        API.lock().await.send_rpc(rpc).await;
    }
}
