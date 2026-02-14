use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

use crate::error::{Error, Result};
use crate::protocol::{Request, SOCKET_PATH};

pub fn send_commnad(request: &Request) -> Result<()> {
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
