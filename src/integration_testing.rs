
use tokio::time::{Duration, sleep};
use tokio_tungstenite::tungstenite::protocol::Message;
use crate::communication::rpc::RPC;
use tokio::sync::Mutex;

lazy_static::lazy_static! {
    static ref EXPECTED_MESSAGE: Mutex<Message> = Mutex::new(Message::Text("Welcome to the server please add your name".to_string())) ;
}

lazy_static::lazy_static! {
    static ref WATING: Mutex<bool> = Mutex::new(false);
}
pub async fn reseived_message(msg: Message){
    let mut wating = WATING.lock().await;
    if *wating {
        eprintln!("message sent out of order")
    }
    assert!(msg == *EXPECTED_MESSAGE.lock().await, "this is not the expected message:"); 
    *wating = false;
}
pub async fn await_message (msg: Message){
    let mut wating = WATING.lock().await;
    while *wating {
        drop(wating);
        sleep(Duration::from_secs(1)).await;
        wating = WATING.lock().await;
    }
    *wating = true;
    let mut expected_message = EXPECTED_MESSAGE.lock().await;
    *expected_message = msg;

}

include!( "../test/SERVER_TEST_CODE_SNIPPET.rs" ); // this is for testing
