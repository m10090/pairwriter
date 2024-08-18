
pub async fn run_test() {
    sleep(Duration::from_secs(6)).await;
    println!("test_is_running");

    let rpc = RPC::CreateFile { path: "./this.c".to_string() };
    let message = rpc.encode().unwrap();
    await_message(message.clone()).await;

    let _ = crate::client::messaging::client_send_message(message).await;
    
    loop {
        let wating = WATING.lock().await;
        if !*wating {
            break;
        }
        drop(wating);
        sleep(Duration::from_secs(1)).await;
    }
    println!("Test Passed!");

}
