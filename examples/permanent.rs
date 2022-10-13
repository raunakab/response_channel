#[tokio::main]
async fn main() {
    type Message = u8;
    type Response = bool;
    const BUFFER_SIZE: usize = 100;

    let (mut tx, mut rx) = response_channel::permanent::channel::<Message, Response>(BUFFER_SIZE, None);

    tokio::task::spawn(async move {
        for i in 0..10 {
            let response = tx.send_await_automatic(i).await.unwrap().unwrap();
            assert_eq!(response, i >= 5);
        };
    });

    while let Some((message, tx)) = rx.recv().await {
        let response = message >= 5;
        tx.send(response).await.unwrap();
    };
}
