# Response Channel
A simple wrapper around the [`tokio::sync::mpsc`](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html) channel to provide a bidirectional response channel.

Usually, instantiating a bidrectional channel is cumbersome and requires a lot of boilerplate.
`response_channel` provides a method to create a bidirectional response channel using the same API as creating a regular [`tokio::sync::mpsc`](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html) channel (i.e., by just calling a function).

```rust
use tokio::task::spawn;
use tokio::join;

type Message = u8;
type Response = u8;

fn main() {
    const BUFFER_SIZE: usize = 10;
    let (mut tx, mut rx) = response_channel::channel::<Message, Response>(BUFFER_SIZE, None);
    let fut1 = spawn(async move {
        for i in 0..10 {
            let response = tx.send_await_automatic(i).await.unwrap().unwrap();
            assert_eq!(response, i + 1);
        };
    });
    let fut2 = spawn(async move {
        while let Some((message, tx)) = rx.recv().await {
            let response = message + 1;
            tx.send(response).await.unwrap();
        };
    });
    let (res1, res2) = join!(fut1, fut2);
    res1.unwrap();
    res2.unwrap();
}
```

## Implementation Notes
Multiple `response_channel::Sender`s can also exist, each one able to send its own messages to the [`Receiver`](https://docs.rs/tokio/latest/tokio/sync/mpsc/struct.Receiver.html) *and receive localized responses*!
Namely, if one particular `response_channel::Sender` sends a message to the [`Receiver`](https://docs.rs/tokio/latest/tokio/sync/mpsc/struct.Receiver.html), other `response_channel::Sender`s will *not* be able to see the response.

The `response_channel::Sender` struct internally contains the forwards transmission line, as well as the reverse transmission and reverse receiving line.
The reason why the reverse transmission line needs to be held is so that the `response_channel::Sender` can send the transmission line as an argument.
The [`Receiver`](https://docs.rs/tokio/latest/tokio/sync/mpsc/struct.Receiver.html) can then take the message (which contains the data and the reverse transmissions line), handle the data, and respond using the given transmission line.

Note that cloning the `response_channel::Sender` will clone the internal [`tokio::sync::mpsc::Sender`](https://docs.rs/tokio/latest/tokio/sync/mpsc/struct.Sender.html) (so that the same [`Receiver`](https://docs.rs/tokio/latest/tokio/sync/mpsc/struct.Receiver.html) will be hit).
It, however, will create a new reverse channel.
This makes sense since a new channel is being created, so a new reverse channel needs to be created.
