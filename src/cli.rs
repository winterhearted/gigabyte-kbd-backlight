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
