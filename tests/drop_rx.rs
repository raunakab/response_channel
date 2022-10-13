#[tokio::test]
async fn main() {
    let (tx, rx) = response_channel::temporary::channel::<u8, u8>(10);
    drop(rx);
    let actual = tx.send_await_automatic(10).await.unwrap_err();
    let expected = response_channel::error::Error::SendError(
        tokio::sync::mpsc::error::SendError(10),
    );
    assert_eq!(actual, expected);
}
