pub async fn run_test(){
    loop {
        let empty = crate::server::connection::is_queue_empty().await;
        if !empty {
            break;
        }
        sleep(Duration::from_secs(1)).await;
    }
    println!("test is running");
    let config = bincode::config::standard();
    let rpc = RPC::SendFile{ file: vec![1,2,3,34,45,6] };
    let message = Message::binary(bincode::encode_to_vec(&rpc,config).unwrap());

    // awaiting message is only for the client and not for the server

    crate::server::connection::server_send_message(message).await;

    let rpc = RPC::MoveCursor{ position: 0, path: "~/this.c".to_string(),line:9};
    let message = Message::binary(bincode::encode_to_vec(&rpc,config).unwrap());

    crate::server::connection::server_send_message(message).await;
    
    println!("test is done");

}
