#[cfg(test)]
mod tests;

use derive_more::Display;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::oneshot::error::RecvError;

#[derive(Display)]
/// The error type.
///
/// Since only [`tokio::sync::mpsc::error::SendError`] and
/// [`tokio::sync::oneshot::error::RecvError`] are the only possible errors,
/// [`Error`] just wraps these in an enum.
pub enum Error<M> {
    /// The [`tokio::sync::mpsc::error::SendError`] variant.
    ///
    /// Occurs if an error in sending the initial message occurs.
    #[display(fmt = "(mpsc) Send Error: {}", _0)]
    SendError(SendError<M>),

    /// The [`tokio::sync::oneshot::error::RecvError`] variant.
    ///
    /// Occurs if an error in receiving the response occurs.
    #[display(fmt = "(oneshot) Receive Error: {}", _0)]
    RecvError(RecvError),
}

impl<M> PartialEq for Error<M>
where
    M: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::SendError(SendError(err)),
                Self::SendError(SendError(other_err)),
            ) => err.eq(other_err),
            (Self::RecvError(err), Self::RecvError(other_err)) => {
                err.eq(other_err)
            },
            _ => false,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl<M> Eq for Error<M> where M: Eq {}

impl<M> From<SendError<M>> for Error<M> {
    fn from(err: SendError<M>) -> Self {
        Self::SendError(err)
    }
}

impl<M> From<RecvError> for Error<M> {
    fn from(err: RecvError) -> Self {
        Self::RecvError(err)
    }
}

impl<M> std::error::Error for Error<M> {}

impl<M> std::fmt::Debug for Error<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
