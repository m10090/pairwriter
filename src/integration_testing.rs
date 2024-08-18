use std::ops::Deref;

// this testing method is not that great and it needs alot of refactoring and cleaning
// todo: use mspc to defined expected messages
use crate::communication::rpc::RPC;
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::tungstenite::protocol::Message;

lazy_static! {
    static ref EXPECTED_MESSAGE: Mutex<Message> = Mutex::new(Message::Text(
        "Welcome to the server please add your name".to_string()
    ));
}

lazy_static! {
    static ref WATING: Mutex<bool> = Mutex::new(true);
}
pub async fn reseived_message(msg: Message) {
    let mut wating = WATING.lock().await;
    if !*wating {
        panic!("message sent out of order")
    }
    let expected_message = EXPECTED_MESSAGE.lock().await;
    assert!(
        msg == *expected_message,
        "this is not the expected message: message found {}, expected: {}",
        msg,
        *expected_message
    );
    *wating = false;
}
pub async fn await_message(msg: Message) {
    loop {
        let wating = WATING.lock().await;
        if !*wating {
            break;
        }
        drop(wating);
        sleep(Duration::from_secs(1)).await;
    }
    let mut wating = WATING.lock().await;
    *wating = true;
    let mut expected_message = EXPECTED_MESSAGE.lock().await;
    *expected_message = msg;
}

include!("../test/test_injection.rs"); // this is for testing
