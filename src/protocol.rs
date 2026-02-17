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

use crate::error::Error;

pub const SOCKET_PATH: &str = "/run/kbdlight.sock";
pub const GROUP_NAME: &str = "kbdlight";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    On,
    Off,
    SetColor { r: u8, g: u8, b: u8 },
    Brightness(u8),
    AdjustBrightness(i8),
}

impl Request {
    pub fn parse(line: &str) -> Result<Self, Error> {
        let line = line.trim();
        let mut parts = line.split_whitespace();
        let cmd = parts
            .next()
            .ok_or_else(|| Error::Protocol("empty command".into()))?;

        match cmd {
            "ON" => Ok(Request::On),
            "OFF" => Ok(Request::Off),
            "COLOR" => {
                let r: u8 = parts
                    .next()
                    .ok_or_else(|| Error::Protocol("missing R".into()))?
                    .parse()
                    .map_err(|_| Error::Protocol("invalid R".into()))?;
                let g: u8 = parts
                    .next()
                    .ok_or_else(|| Error::Protocol("missing G".into()))?
                    .parse()
                    .map_err(|_| Error::Protocol("invalid G".into()))?;
                let b: u8 = parts
                    .next()
                    .ok_or_else(|| Error::Protocol("missing B".into()))?
                    .parse()
                    .map_err(|_| Error::Protocol("invalid B".into()))?;
                Ok(Request::SetColor { r, g, b })
            }
            "BRIGHTNESS" => {
                let level: u8 = parts
                    .next()
                    .ok_or_else(|| Error::Protocol("missing level".into()))?
                    .parse()
                    .map_err(|_| Error::Protocol("invalid level".into()))?;
                Ok(Request::Brightness(level))
            }
            "ADJUST" => {
                let delta: i8 = parts
                    .next()
                    .ok_or_else(|| Error::Protocol("missing delta".into()))?
                    .parse()
                    .map_err(|_| Error::Protocol("invalid delta".into()))?;
                Ok(Request::AdjustBrightness(delta))
            }
            _ => Err(Error::Protocol(format!("unknown command: {cmd}"))),
        }
    }

    pub fn to_line(&self) -> String {
        match self {
            Request::On => "ON".into(),
            Request::Off => "OFF".into(),
            Request::SetColor { r, g, b } => format!("COLOR {r} {g} {b}"),
            Request::Brightness(level) => format!("BRIGHTNESS {level}"),
            Request::AdjustBrightness(delta) => format!("ADJUST {delta}"),
        }
    }
}

pub fn parse_hex_color(s: &str) -> Result<(u8, u8, u8), Error> {
    let s = s.strip_prefix('#').unwrap_or(s);
    if s.len() != 6 {
        return Err(Error::InvalidColor(s.into()));
    }
    let r = u8::from_str_radix(&s[0..2], 16).map_err(|_| Error::InvalidColor(s.into()))?;
    let g = u8::from_str_radix(&s[2..4], 16).map_err(|_| Error::InvalidColor(s.into()))?;
    let b = u8::from_str_radix(&s[4..6], 16).map_err(|_| Error::InvalidColor(s.into()))?;
    Ok((r, g, b))
}

pub fn ok_info_response(msg: &str) -> String {
    format!("OK {msg}\n")
}

pub fn err_response(msg: &str) -> String {
    format!("ERR {msg}\n")
}
