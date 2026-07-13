use snafu::prelude::*;
use std::time::Duration;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    UnknownSysEx { code: u8 },
    BadByte { byte: u8 },
    StdIoError { source: std::io::Error },
    Utf8Error { source: std::str::Utf8Error },
    MessageTooShort,
    PinOutOfBounds { pin: u8, max: u8 },
    Timeout { duration: Duration },
    UnexpectedMessage { expected: &'static str },
    HandshakeFailed { reason: String },
    UnsupportedMode { pin: u8, mode: u8 },
}

impl Error {
    pub fn is_transient(&self) -> bool {
        match self {
            Error::StdIoError { source } => matches!(
                source.kind(),
                std::io::ErrorKind::TimedOut
                    | std::io::ErrorKind::WouldBlock
                    | std::io::ErrorKind::Interrupted
            ),
            _ => false,
        }
    }
}

impl From<backoff::Error<Error>> for Error {
    fn from(value: backoff::Error<Error>) -> Self {
        match value {
            backoff::Error::Permanent(err) => err,
            backoff::Error::Transient { err, .. } => err,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
