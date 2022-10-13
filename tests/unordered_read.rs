use tokio::join;
use tokio::task::spawn;

#[tokio::test]
async fn main() {
    let (mut tx, mut rx) = response_channel::channel::<u8, u8>(10, None);
    let fut1 = spawn(async move {
        let range = 0..10;
        for i in range.clone() {
            tx.send_await(i).await.unwrap();
        }
        for i in range {
            let response = tx.recv().await.unwrap();
            assert_eq!(response, i + 1);
        }
    });
    let fut2 = spawn(async move {
        while let Some((message, tx)) = rx.recv().await {
            let response = message + 1;
            tx.send(response).await.unwrap();
        }
    });
    let (res1, res2) = join!(fut1, fut2);
    res1.unwrap();
    res2.unwrap();
}
