use response_channel::error::Error;
use tokio::sync::mpsc;

#[tokio::test]
async fn main() {
    let (mut tx, rx) = response_channel::channel::<u8, u8>(10, None);
    drop(rx);
    let err = tx.send_await_automatic(1).await.unwrap_err();
    assert!(matches!(err, Error::SendError(mpsc::error::SendError(1))));
}
