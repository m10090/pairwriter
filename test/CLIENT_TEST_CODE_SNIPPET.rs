
pub async fn run_test() {
    sleep(Duration::from_secs(6)).await;
    println!("test_is_running");
    let config = bincode::config::standard();

    let rpc = RPC::CreateFile { path: "~/this".to_string() };
    let message = Message::binary(bincode::encode_to_vec(&rpc,config ).unwrap());
    await_message(message.clone()).await;

    let _ = crate::client::messaging::client_send_message(message).await;

    let rpc = RPC::DeleteFile  { path: "~/this".to_string() };
    let message = Message::binary(bincode::encode_to_vec(&rpc,config ).unwrap());
    await_message(message.clone()).await;
    let _ = crate::client::messaging::client_send_message(message).await;
println!("test is done");

}
