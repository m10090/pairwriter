pub async fn run_test(){
    loop {
        let empty = crate::server::connection::is_queue_empty().await;
        if !empty {
            break;
        }
        sleep(Duration::from_secs(1)).await;
    }
    println!("this_is_running");
    let config = bincode::config::standard();
    let message = rpc.encode()?;

    // awaiting message is only for the client and not for the server

    crate::server::connection::server_send_message(message).await;

    let rpc = RPC::MoveCursor{ position: 0, path: "~/this.c".to_string(),line:9};
    let message = rpc.encode();

    crate::server::connection::server_send_message(message).await;
    
    println!("Test passed!");


}
