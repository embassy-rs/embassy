#[cfg(feature = "std")]
use core::convert::From;
#[cfg(feature = "std")]
use futures::io;

/// Categories of errors that can occur.
///
/// This list is intended to grow over time and it is not recommended to
/// exhaustively match against it.
#[derive(defmt::Format, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// An entity was not found, often a file.
    NotFound,
    /// The operation lacked the necessary privileges to complete.
    PermissionDenied,
    /// The connection was refused by the remote server.
    ConnectionRefused,
    /// The connection was reset by the remote server.
    ConnectionReset,
    /// The connection was aborted (terminated) by the remote server.
    ConnectionAborted,
    /// The network operation failed because it was not connected yet.
    NotConnected,
    /// A socket address could not be bound because the address is already in
    /// use elsewhere.
    AddrInUse,
    /// A nonexistent interface was requested or the requested address was not
    /// local.
    AddrNotAvailable,
    /// The operation failed because a pipe was closed.
    BrokenPipe,
    /// An entity already exists, often a file.
    AlreadyExists,
    /// The operation needs to block to complete, but the blocking operation was
    /// requested to not occur.
    WouldBlock,
    /// A parameter was incorrect.
    InvalidInput,
    /// Data not valid for the operation were encountered.
    ///
    /// Unlike [`InvalidInput`], this typically means that the operation
    /// parameters were valid, however the error was caused by malformed
    /// input data.
    ///
    /// For example, a function that reads a file into a string will error with
    /// `InvalidData` if the file's contents are not valid UTF-8.
    ///
    /// [`InvalidInput`]: #variant.InvalidInput
    InvalidData,
    /// The I/O operation's timeout expired, causing it to be canceled.
    TimedOut,
    /// An error returned when an operation could not be completed because a
    /// call to [`write`] returned [`Ok(0)`].
    ///
    /// This typically means that an operation could only succeed if it wrote a
    /// particular number of bytes but only a smaller number of bytes could be
    /// written.
    ///
    /// [`write`]: ../../std/io/trait.Write.html#tymethod.write
    /// [`Ok(0)`]: ../../std/io/type.Result.html
    WriteZero,
    /// This operation was interrupted.
    ///
    /// Interrupted operations can typically be retried.
    Interrupted,

    /// An error returned when an operation could not be completed because an
    /// "end of file" was reached prematurely.
    ///
    /// This typically means that an operation could only succeed if it read a
    /// particular number of bytes but only a smaller number of bytes could be
    /// read.
    UnexpectedEof,

    /// An operation would have read more data if the given buffer was large.
    ///
    /// This typically means that the buffer has been filled with the first N bytes
    /// of the read data.
    Truncated,

    /// Any I/O error not part of this list.
    Other,
}

pub type Result<T> = core::result::Result<T, Error>;

#[cfg(feature = "std")]
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        match err.kind() {
            io::ErrorKind::NotFound => Error::NotFound,
            io::ErrorKind::PermissionDenied => Error::PermissionDenied,
            io::ErrorKind::ConnectionRefused => Error::ConnectionRefused,
            io::ErrorKind::ConnectionReset => Error::ConnectionReset,
            io::ErrorKind::ConnectionAborted => Error::ConnectionAborted,
            io::ErrorKind::NotConnected => Error::NotConnected,
            io::ErrorKind::AddrInUse => Error::AddrInUse,
            io::ErrorKind::AddrNotAvailable => Error::AddrNotAvailable,
            io::ErrorKind::BrokenPipe => Error::BrokenPipe,
            io::ErrorKind::AlreadyExists => Error::AlreadyExists,
            io::ErrorKind::WouldBlock => Error::WouldBlock,
            io::ErrorKind::InvalidInput => Error::InvalidInput,
            io::ErrorKind::InvalidData => Error::InvalidData,
            io::ErrorKind::TimedOut => Error::TimedOut,
            io::ErrorKind::WriteZero => Error::WriteZero,
            io::ErrorKind::Interrupted => Error::Interrupted,
            io::ErrorKind::UnexpectedEof => Error::UnexpectedEof,
            _ => Error::Other,
        }
    }
}

//#[cfg(feature = "std")]
//impl std::error::Error for Error {}

/*
impl From<smoltcp::Error> for Error {
    fn from(err: smoltcp::Error) -> Error {
        match err {
            smoltcp::Error::Exhausted => Error::Exhausted,
            smoltcp::Error::Illegal => Error::Illegal,
            smoltcp::Error::Unaddressable => Error::Unaddressable,
            smoltcp::Error::Truncated => Error::Truncated,
            smoltcp::Error::Checksum => Error::Checksum,
            smoltcp::Error::Unrecognized => Error::Unrecognized,
            smoltcp::Error::Fragmented => Error::Fragmented,
            smoltcp::Error::Malformed => Error::Malformed,
            smoltcp::Error::Dropped => Error::Dropped,
            _ => Error::Other,
        }
    }
}
*/
