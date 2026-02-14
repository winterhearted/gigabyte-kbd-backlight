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
    Daemon,
    On,
    Off,
    SetColor {
        color: String,
    },
    SetBrightness {
        level: u8,
    },
    AdjustBrightness {
        #[arg(allow_hyphen_values = true)]
        delta: String,
    },
}
