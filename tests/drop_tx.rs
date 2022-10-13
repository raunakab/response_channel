#[tokio::test]
async fn main() {
    let (tx, mut rx) = response_channel::channel::<u8, u8>(10, None);
    drop(tx);
    let message = rx.recv().await;
    assert!(matches!(message, None));
}
