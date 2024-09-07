pub async fn run_test(){
    loop {
        let empty = crate::server::connection::no_client_connected().await;
        if !empty {
            break;
        }
        sleep(Duration::from_secs(1)).await;
    }
    println!("this_is_running");

    // awaiting message is only for the client and not for the server


    let rpc = RPC::CreateFile { path: "./this.c".to_string() };
    let message = rpc.encode();

    crate::server::messageing::server_send_message(message.unwrap()).await;
    
    println!("Test passed!");


}
