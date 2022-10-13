use tokio::sync::mpsc;

use crate::error::Error;

#[test]
fn send_error_eq() {
    let a = Error::SendError(mpsc::error::SendError(10));
    let b = Error::SendError(mpsc::error::SendError(10));
    let c = Error::SendError(mpsc::error::SendError(10));

    // reflexive
    assert_eq!(a, a);

    // symmetric
    assert_eq!(a, b);
    assert_eq!(b, a);

    // transitive
    assert_eq!(a, b);
    assert_eq!(b, c);
    assert_eq!(a, c);
}
