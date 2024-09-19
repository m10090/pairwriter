pub async fn run_test() {
    // await_undefined_message().await;
    sleep(Duration::from_secs(6)).await;
    use std::fs;
    println!("test_is_running");

    let client_api = crate::client::API.get().unwrap();

    // await_message(message.clone()).await;

    let rpc = RPC::CreateFile {
        path: "./test.txt".to_string(),
    };

    set_and_await(Some( rpc.encode().unwrap() )).await;
    let _ = client_api.lock().await.send_rpc(rpc).await;
    
    set_and_await(None).await;
    if fs::read_to_string("../test.txt").is_ok()  {
        println!("file created");
    } else {
        panic!("file not created");
    }
    let _ = client_api
        .lock()
        .await
        .read_file("./test.txt".to_string())
        .await;


    set_and_await(None).await;
    println!("got the file");

    let _ = client_api
        .lock()
        .await
        .edit_buf("./test.txt".to_string(), Some(0), Some(0), "test")
        .await;


    set_and_await(
        Some( RPC::FileSaved {
            path: "./test.txt".to_string(),
        }
        .encode()
        .unwrap(), )
    ).await;
    println!("edited the file");

    let _ = client_api
        .lock()
        .await
        .send_rpc(RPC::ReqSaveFile {
            path: "./test.txt".to_string(),
        })
        .await;
    await_the_last_message().await;

    println!("file saved");
    if fs::read_to_string("../test.txt").unwrap() == "test" {
        let _ = fs::remove_file("../test.txt");
        println!("Test Passed!");
    } else {
        println!("Test Failed!");
    }
}
