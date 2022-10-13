#[cfg(test)]
mod tests;

use std::ops::Deref;

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::error::Error;

/// Creates a temporary, bidirectional response channel (using
/// [`tokio::sync::oneshot`] as a temporary, reverse channel).
///
/// ### Arguments:
/// - `buffer`: The size of the forward channel.
///
/// ### Notes:
/// The returned type is a 2-tuple in which the first element is the
/// transmission end of the channel and the second is the receiving end.
///
/// As previously noted, the reverse channel that is instantiated to allow the
/// receiver to respond to the sender is a simple [`tokio::sync::oneshot`]
/// channel. This implies that, for every message that is sent, a new
/// [`tokio::sync::oneshot`] will need to be instantiated. This is permissible
/// if messages (and thus responses to the according message) are far and few.
///
/// If this is *not* the case, consider using (...).
///
/// ### Examples:
/// ```rust
/// # tokio_test::block_on(async {
/// const BUFFER_SIZE: usize = 100;
///
/// let (mut tx, mut rx) = response_channel::temporary::channel::<u8, bool>(BUFFER_SIZE);
///
/// tokio::task::spawn(async move {
///     for i in 0..10 {
///         let response = tx.send_await_automatic(i).await.unwrap();
///         assert_eq!(response, i >= 5);
///     };
/// });
///
/// while let Some((message, tx)) = rx.recv().await {
///     let response = message >= 5;
///     tx.send(response).unwrap();
/// };
/// # });
/// ```
pub fn channel<M, R>(
    buffer: usize,
) -> (Sender<M, R>, mpsc::Receiver<(M, oneshot::Sender<R>)>) {
    let (tx, rx) = mpsc::channel(buffer);
    (Sender(tx), rx)
}

#[derive(Clone)]
#[cfg_attr(not(release), derive(Debug))]
/// ...
pub struct Sender<M, R>(pub(crate) mpsc::Sender<(M, oneshot::Sender<R>)>);

impl<M, R> Sender<M, R> {
    /// ...
    pub async fn send_await(
        &self,
        message: M,
    ) -> Result<oneshot::Receiver<R>, mpsc::error::SendError<M>> {
        let (tx, rx) = oneshot::channel::<R>();
        self.0
            .send((message, tx))
            .await
            .map(|()| rx)
            .map_err(|mpsc::error::SendError((m, _))| mpsc::error::SendError(m))
    }

    /// ...
    pub async fn send_await_automatic(
        &self,
        message: M,
    ) -> Result<R, Error<M>> {
        let response = self.send_await(message).await?.await?;
        Ok(response)
    }
}

impl<M, R> Deref for Sender<M, R> {
    type Target = mpsc::Sender<(M, oneshot::Sender<R>)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
