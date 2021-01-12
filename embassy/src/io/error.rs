/// Categories of errors that can occur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
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
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        match err.kind() {
            std::io::ErrorKind::NotFound => Error::NotFound,
            std::io::ErrorKind::PermissionDenied => Error::PermissionDenied,
            std::io::ErrorKind::ConnectionRefused => Error::ConnectionRefused,
            std::io::ErrorKind::ConnectionReset => Error::ConnectionReset,
            std::io::ErrorKind::ConnectionAborted => Error::ConnectionAborted,
            std::io::ErrorKind::NotConnected => Error::NotConnected,
            std::io::ErrorKind::AddrInUse => Error::AddrInUse,
            std::io::ErrorKind::AddrNotAvailable => Error::AddrNotAvailable,
            std::io::ErrorKind::BrokenPipe => Error::BrokenPipe,
            std::io::ErrorKind::AlreadyExists => Error::AlreadyExists,
            std::io::ErrorKind::WouldBlock => Error::WouldBlock,
            std::io::ErrorKind::InvalidInput => Error::InvalidInput,
            std::io::ErrorKind::InvalidData => Error::InvalidData,
            std::io::ErrorKind::TimedOut => Error::TimedOut,
            std::io::ErrorKind::WriteZero => Error::WriteZero,
            std::io::ErrorKind::Interrupted => Error::Interrupted,
            std::io::ErrorKind::UnexpectedEof => Error::UnexpectedEof,
            _ => Error::Other,
        }
    }
}

#[cfg(feature = "std")]
impl From<Error> for std::io::Error {
    fn from(e: Error) -> Self {
        let kind = match e {
            Error::NotFound => std::io::ErrorKind::NotFound,
            Error::PermissionDenied => std::io::ErrorKind::PermissionDenied,
            Error::ConnectionRefused => std::io::ErrorKind::ConnectionRefused,
            Error::ConnectionReset => std::io::ErrorKind::ConnectionReset,
            Error::ConnectionAborted => std::io::ErrorKind::ConnectionAborted,
            Error::NotConnected => std::io::ErrorKind::NotConnected,
            Error::AddrInUse => std::io::ErrorKind::AddrInUse,
            Error::AddrNotAvailable => std::io::ErrorKind::AddrNotAvailable,
            Error::BrokenPipe => std::io::ErrorKind::BrokenPipe,
            Error::AlreadyExists => std::io::ErrorKind::AlreadyExists,
            Error::WouldBlock => std::io::ErrorKind::WouldBlock,
            Error::InvalidInput => std::io::ErrorKind::InvalidInput,
            Error::InvalidData => std::io::ErrorKind::InvalidData,
            Error::TimedOut => std::io::ErrorKind::TimedOut,
            Error::WriteZero => std::io::ErrorKind::WriteZero,
            Error::Interrupted => std::io::ErrorKind::Interrupted,
            Error::UnexpectedEof => std::io::ErrorKind::UnexpectedEof,
            Error::Truncated => std::io::ErrorKind::Other,
            Error::Other => std::io::ErrorKind::Other,
        };
        std::io::Error::new(kind, "embassy::io::Error")
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
