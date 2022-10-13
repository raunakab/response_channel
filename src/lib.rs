#![deny(missing_docs)]

//! # Response Channel:
//! A wrapper crate (around the [`tokio::sync::mpsc`] channels) allowing for
//! bidirectional communication. One can simply create a bidrectional channel by
//! hand and manage all of the boilerplate in setting it up and handling
//! responses, but that gets cumbersome quite quickly (especially if many
//! bidirectional channels are required).
//!
//! This crate allows the developer to create a simple bidirectional
//! response channel using the same API as creating a [`tokio::sync::mpsc`]
//! channel (i.e., just a simple function call).
//!
//! ### Example:
//! ```rust
//! # tokio_test::block_on(async {
//! const BUFFER_SIZE: usize = 10;
//!
//! type Message = u8;
//! type Response = bool;
//!
//! let (mut tx, mut rx) = response_channel::channel::<Message, Response>(BUFFER_SIZE);
//!
//! tokio::task::spawn(async move {
//!     // send the initial message and await for a response.
//!     let response = tx.send_await_automatic(100).await.unwrap().unwrap();
//!     assert!(response);
//! });
//!
//! // receive a message and destructure it into the actual message and reverse transmission line
//! // (the reverse transmission line is how you send the response back to the caller!)
//! let (message, mut reverse_tx) = rx.recv().await.unwrap().unwrap();
//! let response = message >= 5;
//! reverse_tx.send(response).await.unwrap();
//! # });
//! ```

/// The error type for this crate.
pub mod error;

use std::ops::Deref;

use tokio::sync::mpsc;

/// Creates a new permament, bidirectional response channel.
///
/// ### Notes:
/// The reverse channel is implemented using a [`tokio::sync::mpsc`] channel,
/// which means that it can support multiple messages. Namely, it does not need
/// to be reinstantiated after a response has been sent.
///
/// An important thing to note is that [`Sender`] *can* be cloned!
/// When cloning occurs, the same forward transmission line is cloned to the new
/// struct, *but a new response channel is created*. This means that messages
/// are local to the specific sender!
///
/// ### Arguments:
/// - `buffer`: The size of the forward channel.
/// - `reverse_buffer`: The size of the reverse channel. If this is [`None`],
///   `buffer` will be used.
///
/// ### Examples:
/// ```rust
/// # tokio_test::block_on(async {
/// const BUFFER_SIZE: usize = 100;
///
/// let (mut tx, mut rx) = response_channel::channel::<u8, bool>(BUFFER_SIZE, None);
///
/// tokio::task::spawn(async move {
///     for i in 0..10 {
///         let response = tx.send_await_automatic(i).await.unwrap().unwrap();
///         assert_eq!(response, i >= 5);
///     };
/// });
///
/// while let Some((message, tx)) = rx.recv().await {
///     let response = message >= 5;
///     tx.send(response).await.unwrap();
/// };
/// # });
/// ```
pub fn channel<M, R>(
    buffer: usize,
    reverse_buffer: Option<usize>,
) -> (Sender<M, R>, mpsc::Receiver<(M, mpsc::Sender<R>)>) {
    let (tx, rx) = mpsc::channel(buffer);
    let (reverse_tx, reverse_rx) =
        mpsc::channel(reverse_buffer.unwrap_or(buffer));
    (
        Sender {
            tx,
            reverse_tx,
            reverse_rx,
        },
        rx,
    )
}

/// The [`Sender`] type which contains the necessary information to provide a
/// bidirectional response channel.
#[cfg_attr(not(release), derive(Debug))]
pub struct Sender<M, R> {
    pub(crate) tx: mpsc::Sender<(M, mpsc::Sender<R>)>,
    pub(crate) reverse_tx: mpsc::Sender<R>,
    pub(crate) reverse_rx: mpsc::Receiver<R>,
}

impl<M, R> Sender<M, R> {
    /// Sends the given message to the receiver.
    ///
    /// ### Arguments:
    /// - `message`: The message that needs to be sent.
    ///
    /// ### Notes:
    /// This function does *not* try to receive the response!
    /// The user must do this explicitly if they are required to read the
    /// response.
    ///
    /// ### Example:
    /// ```rust
    /// # tokio_test::block_on(async {
    /// type Message = u8;
    /// type Response = bool;
    /// # let (mut tx, rx) = response_channel::channel::<Message, Response>(100, None);
    ///
    /// // sends the first message (but does not eagerly await the response!)
    /// tx.send_await(10).await.unwrap();
    ///
    /// // sends another message (but once again does not eagerly await the response!)
    /// tx.send_await(11).await.unwrap();
    /// # drop(rx);
    /// # });
    /// ```
    ///
    /// If you wish read the responses, please refer to [`Sender::recv`].
    pub async fn send_await(&self, message: M) -> Result<(), error::Error<M>> {
        self.tx
            .send((message, self.reverse_tx.clone()))
            .await
            .map_err(|mpsc::error::SendError((m, _))| {
                mpsc::error::SendError(m)
            })?;
        Ok(())
    }

    /// Sends the given message to the receiver.
    ///
    /// ### Arguments:
    /// - `message`: The message that needs to be sent.
    ///
    /// ### Notes:
    /// This function will send the message and then *automatically listen for
    /// the response right away*. It is equivalent to calling
    /// [`Sender::send_await`] followed immediately by [`Sender::recv`].
    ///
    /// ### Example:
    /// ```rust
    /// # tokio_test::block_on(async move {
    /// // for example, consider the two type aliases:
    /// type Message = u8;
    /// type Response = u8;
    /// # let (mut tx, mut rx) = response_channel::channel::<Message, Response>(1, None);
    ///
    /// let fut1 = tokio::task::spawn(async move {
    ///     let message = 100;
    ///     let response = tx.send_await_automatic(message).await.unwrap().unwrap();
    ///     assert_eq!(response, message + 1);
    /// });
    ///
    /// let fut2 = tokio::task::spawn(async move {
    ///     while let Some((message, tx)) = rx.recv().await {
    ///         let response = message + 1;
    ///         tx.send(response).await.unwrap();
    ///     };
    /// });
    ///
    /// # let (res1, res2) = tokio::join!(fut1, fut2);
    /// # res1.unwrap();
    /// # res2.unwrap();
    /// # });
    /// ```
    ///
    /// This is usually the most common usage (i.e., waiting for the response
    /// right away).
    pub async fn send_await_automatic(
        &mut self,
        message: M,
    ) -> Result<Option<R>, error::Error<M>> {
        self.send_await(message).await?;
        let response = self.reverse_rx.recv().await;
        Ok(response)
    }

    /// Receives a message from the reverse channel.
    ///
    /// ### Example:
    /// ```rust
    /// # tokio_test::block_on(async move {
    /// // for example, consider the two type aliases:
    /// type Message = u8;
    /// type Response = u8;
    /// # let (mut tx, mut rx) = response_channel::channel::<Message, Response>(1, None);
    ///
    /// let fut1 = tokio::task::spawn(async move {
    ///     let message1 = 100;
    ///     let message2 = 100;
    ///     tx.send_await(message1).await.unwrap();
    ///     tx.send_await(message2).await.unwrap();
    ///
    ///     let response1 = tx.recv().await.unwrap();
    ///     let response2 = tx.recv().await.unwrap();
    ///     assert_eq!(response1, message1 + 1);
    ///     assert_eq!(response2, message2 + 1);
    /// });
    ///
    /// let fut2 = tokio::task::spawn(async move {
    ///     while let Some((message, tx)) = rx.recv().await {
    ///         let response = message + 1;
    ///         tx.send(response).await.unwrap();
    ///     };
    /// });
    ///
    /// # let (res1, res2) = tokio::join!(fut1, fut2);
    /// # res1.unwrap();
    /// # res2.unwrap();
    /// # });
    /// ```
    pub async fn recv(&mut self) -> Option<R> {
        self.reverse_rx.recv().await
    }
}

impl<M, R> Clone for Sender<M, R> {
    fn clone(&self) -> Self {
        let reverse_buffer = self.reverse_tx.max_capacity();
        let (reverse_tx, reverse_rx) = mpsc::channel(reverse_buffer);
        Self {
            tx: self.tx.clone(),
            reverse_tx,
            reverse_rx,
        }
    }
}

impl<M, R> Deref for Sender<M, R> {
    type Target = mpsc::Sender<(M, mpsc::Sender<R>)>;

    fn deref(&self) -> &Self::Target {
        &self.tx
    }
}
