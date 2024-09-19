

pub async fn run_test() {
    use std::fs;
    sleep(Duration::from_secs(6)).await;
    let client_api = crate::client::API.get().unwrap();
    println!("test_is_running");

    let rpc = RPC::CreateFile { path: "./this.c".to_string() };
    

    set_and_await(Some( rpc.encode().unwrap() )).await;

    let _ = client_api.lock().await.send_rpc(rpc).await;

    sleep(Duration::from_secs(10)).await;
    set_and_await(None).await; // this will wait for the next message
    
    if fs::read_to_string("../this.c").unwrap() == "" {
        println!("Test Passed!");
    } else {
        println!("Test Failed!");
    }
}

