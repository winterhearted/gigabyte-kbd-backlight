// Copyright (C) 2026 Winterhearted

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
