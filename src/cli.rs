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

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "gigabyte-kbd-backlight",
    version,
    about = "Gigabyte keyboard backlight control"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run the privileged daemon
    Daemon,
    /// Turn backlight on (max brightness)
    On,
    /// Turn backlight off
    Off,
    /// Set backlight color (hex: RRGGBB or #RRGGBB)
    SetColor {
        /// Color in RRGGBB or #RRGGBB format
        color: String,
    },
    /// Set brightness level (0-9)
    SetBrightness {
        /// Brightness level (0 = off, 9 = max)
        level: u8,
    },
    /// Adjust brightness by delta (-9 to +9)
    AdjustBrightness {
        /// Delta value (e.g. +3, -2)
        #[arg(allow_hyphen_values = true)]
        delta: String,
    },
}
