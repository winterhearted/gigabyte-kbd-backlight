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
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::{fs, thread};

use nix::unistd::{Group, Uid, chown};

use crate::ec::EcPort;
use crate::error::{Error, Result};
use crate::protocol::{self, GROUP_NAME, Request, SOCKET_PATH};

pub struct Daemon {
    ec: EcPort,
    brightness: u8,
}

impl Daemon {
    pub fn run() -> Result<()> {
        let ec = EcPort::open()?;
        let mut daemon = Daemon { ec, brightness: 9 };

        // Remove stale socket
        let _ = fs::remove_file(SOCKET_PATH);

        let listener = UnixListener::bind(SOCKET_PATH)?;
        listener.set_nonblocking(true)?;

        // Set socket permissions: root:kbdlight, 0660
        Self::set_socket_permissions()?;

        // Register signal handlers
        let shutdown = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&shutdown))?;
        signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&shutdown))?;

        eprintln!("daemon: listening on {SOCKET_PATH}");

        while !shutdown.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _)) => {
                    if let Err(e) = daemon.handle_client(stream) {
                        eprintln!("daemon: client error: {e}");
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    eprintln!("daemon: accept error: {e}");
                }
            }
        }

        eprintln!("daemon: shutting down");
        let _ = fs::remove_file(SOCKET_PATH);
        Ok(())
    }

    fn set_socket_permissions() -> Result<()> {
        let gid = Group::from_name(GROUP_NAME)
            .map_err(|e| Error::Io(e.into()))?
            .map(|g| g.gid);

        chown(SOCKET_PATH, Some(Uid::from_raw(0)), gid).map_err(|e| Error::Io(e.into()))?;

        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(SOCKET_PATH, fs::Permissions::from_mode(0o660))?;

        Ok(())
    }

    fn handle_client(&mut self, stream: UnixStream) -> Result<()> {
        let mut reader = BufReader::new(&stream);
        let mut line = String::new();
        reader.read_line(&mut line)?;

        let response = match Request::parse(&line) {
            Ok(req) => match self.execute(&req) {
                Ok(msg) => msg,
                Err(e) => protocol::err_response(&e.to_string()),
            },
            Err(e) => protocol::err_response(&e.to_string()),
        };

        let mut writer = stream;
        writer.write_all(response.as_bytes())?;
        Ok(())
    }

    fn execute(&mut self, req: &Request) -> Result<String> {
        match req {
            Request::On => {
                self.ec.turn_on()?;
                self.brightness = 9;
                Ok(protocol::ok_info_response("backlight on"))
            }
            Request::Off => {
                self.ec.turn_off()?;
                self.brightness = 0;
                Ok(protocol::ok_info_response("backlight off"))
            }
            Request::SetColor { r, g, b } => {
                self.ec.set_color(*r, *g, *b)?;
                Ok(protocol::ok_info_response(&format!(
                    "color #{r:02X}{g:02X}{b:02X}"
                )))
            }
            Request::Brightness(level) => {
                if *level > 9 {
                    return Err(Error::InvalidBrightness(*level));
                }
                self.ec.set_brightness(*level)?;
                self.brightness = *level;
                Ok(protocol::ok_info_response(&format!(
                    "brightness {}/9",
                    self.brightness
                )))
            }
            Request::AdjustBrightness(delta) => {
                let new = (self.brightness as i16 + *delta as i16).clamp(0, 9) as u8;
                self.ec.set_brightness(new)?;
                self.brightness = new;
                Ok(protocol::ok_info_response(&format!(
                    "brightness {}/9",
                    self.brightness
                )))
            }
        }
    }
}
