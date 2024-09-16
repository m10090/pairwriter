// this testing method is not that great and it needs alot of refactoring and cleaning
// todo: use mspc to defined expected messages
#[allow(unused_imports)] // this is used when injected the integration testing file
use crate::communication::rpc::RPC;
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::tungstenite::protocol::Message;

lazy_static! {
    static ref EXPECTED_MESSAGE: Mutex<Option<Message>> = Mutex::new(None);
    static ref WAITING: Mutex<bool> = Mutex::new(true);
}
pub(crate) async fn reseived_message(msg: Message) {
    let mut waiting = WAITING.lock().await;
    if !*waiting && !msg.is_empty() {
        // if I got a message and I'm not waiting for it, then it's an error
        panic!("message sent out of order")
    }
    let expected_message = EXPECTED_MESSAGE.lock().await;
    assert!(
        expected_message.is_none() || msg == *expected_message.as_ref().unwrap(),
        "this is not the expected message: message found {}, expected: {}",
        msg,
        expected_message.as_ref().unwrap()
    );
    *waiting = false;
}

/// is used to set the expected message and wait for it
/// if the message is None, then it will wait for any message
/// this will block the thread until the expected message is received
pub(crate) async fn set_and_await(msg: Option<Message>) {
    loop {
        let waiting = WAITING.lock().await;
        if !*waiting {
            break;
        }
        drop(waiting);
        sleep(Duration::from_secs(1)).await;
    }
    let mut wating = WAITING.lock().await;
    *wating = true;
    let mut expected_message = EXPECTED_MESSAGE.lock().await;
    *expected_message = msg;
}

pub(crate) async fn await_the_last_message() {
    loop {
        let wating = WAITING.lock().await;
        if !*wating {
            break;
        }
        drop(wating);
        sleep(Duration::from_secs(1)).await;
    }
}

include!("../test/test_injection.rs"); // this is for testing
