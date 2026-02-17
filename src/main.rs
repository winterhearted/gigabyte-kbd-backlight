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

mod cli;
mod client;
mod daemon;
mod ec;
mod error;
mod protocol;

use clap::Parser;

use cli::{Cli, Command};
use error::Error;
use protocol::Request;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli.command) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run(command: Command) -> error::Result<()> {
    match command {
        Command::Daemon => daemon::Daemon::run(),
        cmd => {
            let request = command_to_request(cmd)?;
            client::send_command(&request)
        }
    }
}

fn command_to_request(cmd: Command) -> error::Result<Request> {
    match cmd {
        Command::On => Ok(Request::On),
        Command::Off => Ok(Request::Off),
        Command::SetColor { color } => {
            let (r, g, b) = protocol::parse_hex_color(&color)?;
            Ok(Request::SetColor { r, g, b })
        }
        Command::SetBrightness { level } => {
            if level > 9 {
                return Err(Error::InvalidBrightness(level));
            }
            Ok(Request::Brightness(level))
        }
        Command::AdjustBrightness { delta } => {
            let delta: i8 = delta.parse().map_err(|_| {
                Error::Protocol(format!("invalid delta: {delta} (expected -9 to +9)"))
            })?;
            if !(-9..=9).contains(&delta) {
                return Err(Error::Protocol(format!(
                    "delta out of range: {delta} (expected -9 to +9)"
                )));
            }
            Ok(Request::AdjustBrightness(delta))
        }
        Command::Daemon => unreachable!(),
    }
}
