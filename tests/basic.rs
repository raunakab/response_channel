#[tokio::test]
async fn main() {
    let (tx, mut rx) = response_channel::temporary::channel::<u8, u8>(10);
    tokio::task::spawn(async move {
        while let Some((data, tx)) = rx.recv().await {
            tx.send(data + 1).unwrap();
        }
    });
    let actual = tx.send_await_automatic(10).await.unwrap();
    let expected = 11;
    assert_eq!(actual, expected);
}
