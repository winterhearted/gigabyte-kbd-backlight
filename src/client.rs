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

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

use crate::error::{Error, Result};
use crate::protocol::{Request, SOCKET_PATH};

pub fn send_command(request: &Request) -> Result<()> {
    let mut stream = UnixStream::connect(SOCKET_PATH).map_err(|e| {
        if e.kind() == std::io::ErrorKind::ConnectionRefused
            || e.kind() == std::io::ErrorKind::NotFound
        {
            Error::DaemonNotRunning
        } else if e.kind() == std::io::ErrorKind::PermissionDenied {
            Error::Permission
        } else {
            Error::Io(e)
        }
    })?;

    let line = format!("{}\n", request.to_line());
    stream.write_all(line.as_bytes())?;

    let mut reader = BufReader::new(&stream);
    let mut response = String::new();
    reader.read_line(&mut response)?;

    let response = response.trim();
    if response.starts_with("ERR") {
        let msg = response.strip_prefix("ERR ").unwrap_or(response);
        eprintln!("error: {msg}");
        std::process::exit(1);
    } else {
        let msg = response.strip_prefix("OK ").unwrap_or(response);
        if !msg.is_empty() && msg != "OK" {
            println!("{msg}");
        }
    }

    Ok(())
}
