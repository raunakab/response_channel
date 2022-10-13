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
//! let (mut tx, mut rx) = response_channel::ttemporary::channel::<Message, Response>(BUFFER_SIZE);
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

/// Bidirectional response channels (in which the reverse channel is implemented
/// using [`tokio::sync::mpsc`]).
///
/// ### Notes:
/// Since the reverse channel supports sending multiple messages, it is able to
/// stay alive for the lifetime of the [`crate::permanent::Sender`] that
/// owns it. Therefore, repeated reinstantiation is *not* required.
///
/// This is useful for when a sender and receiver require a long-living
/// bidirectional communications channel.
///
/// ### Example:
/// ```rust
/// # tokio_test::block_on(async move {
/// type Message = u8;
/// type Response = u8;
///
/// const BUFFER_SIZE: usize = 100;
///
/// let (tx, rx) = response_channel::permanent::channel::<Message, Response>(BUFFER_SIZE, None);
/// # });
/// ```
///
/// Notice that the APIs for the creation of a [`crate::permanent`] response
/// channel vs a [`crate::temporary`] response channel are very similar.
/// The only noticeable difference is that the [`crate::permanent`] channel
/// requires a `buffer_size: usize` for the size of its (permanent) reverse
/// channel as well.
pub mod permanent;

/// Bidirectional response channels (in which the reverse channel is implemented
/// using [`tokio::sync::oneshot`]).
///
/// Since the reverse channel can only send 1 message (hence the name
/// [`tokio::sync::oneshot`]), it is consumed after receiving a response.
/// This means that repeated reinstantiation of a reverse channel is needed
/// everytime a message is required to be sent.
///
/// This is useful for when communications between a sender and receiver are
/// limited.
///
/// ### Example:
/// ```rust
/// # tokio_test::block_on(async move {
/// type Message = u8;
/// type Response = u8;
///
/// const BUFFER_SIZE: usize = 100;
///
/// let (tx, rx) = response_channel::temporary::channel::<Message, Response>(BUFFER_SIZE);
/// # });
/// ```
///
/// Notice that the APIs for the creation of a [`crate::permanent`] response
/// channel vs a [`crate::temporary`] response channel are very similar.
/// The only noticeable difference is that the [`crate::permanent`] channel
/// requires a `buffer_size: usize` for the size of its (permanent) reverse
/// channel as well.
pub mod temporary;
