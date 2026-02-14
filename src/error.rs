use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    EcTimeout,
    InvalidBrightness(u8),
    InvalidColor(String),
    Protocol(String),
    Permission,
    DaemonNotRunning,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::EcTimeout => write!(f, "EC timeout (IBF stuck)"),
            Error::InvalidBrightness(v) => write!(f, "invalid brightness level: {v} (expected 0-9)"),
            Error::InvalidColor(s) => write!(f, "invalid color: {s}"),
            Error::Protocol(s) => write!(f, "protocol error: {s}"),
            Error::Permission => write!(f, "permission denied"),
            Error::DaemonNotRunning => write!(f, "daemon not running (cannot connect to socket)"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}
