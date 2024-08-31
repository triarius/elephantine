use clap_serde_derive::ClapSerde;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, time::Duration};

#[allow(clippy::module_name_repetitions)]
#[derive(ClapSerde, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Config {
    /// The X display to use for the dialog.
    #[arg(short = 'D', long, env = "PINENTRY_DISPLAY", value_name = "DISPLAY")]
    pub display: Option<String>,

    /// The tty terminal node name
    #[arg(short = 'T', long, env = "TTYNAME", value_name = "FILE")]
    pub ttyname: Option<String>,

    // The tty terminal type
    #[arg(short = 'N', long, env = "TTYTYPE", value_name = "NAME")]
    pub ttytype: Option<String>,

    /// The `LC_CTYPE` locale category.
    #[arg(short = 'C', long, env = "LC_CTYPE", value_name = "STRING")]
    pub lc_ctype: Option<String>,

    /// The `LC_MESSAGES` value.
    #[arg(short = 'M', long, env = "LC_MESSAGES", value_name = "STRING")]
    pub lc_messages: Option<String>,

    /// Timeout in seconds for requests that show dialogs to the user.
    /// E.g. GETPIN, CONFIRM, etc.
    #[arg(
        short = 'o',
        long,
        env = "ELEPHANTINE_TIMEOUT",
        value_name = "SECS",
        value_parser = parse_duration,
        default_value = "300",
    )]
    pub timeout: Option<Duration>,

    /// Grab keyboard only while the window is focused.
    #[arg(short = 'g', long, env = "ELEPHANTINE_NO_LOCAL_GRAB")]
    pub no_local_grab: bool,

    /// Parent window ID (for partitioning).
    #[arg(short = 'W', long, value_name = "WINDOW_ID")]
    pub parent_wid: Option<String>,

    /// Custom colors for the dialog.
    #[arg(short = 'c', long, value_name = "STRING")]
    pub colors: Option<String>,

    /// The alert mode (none, beep, or flash).
    #[arg(short = 'a', long, value_name = "STRING")]
    pub ttyalert: Option<String>,

    /// The command to run the dialog.
    /// It must print the input to stdout.
    #[arg(
        long,
        value_name = "COMMAND",
        value_delimiter = ' ',
        num_args = 1..,
        default_value = "walker --password",
    )]
    pub command: Vec<String>,
}

fn parse_duration(s: &str) -> Result<Duration> {
    Ok(Duration::from_secs(s.parse::<u64>()?))
}

impl TryFrom<&PathBuf> for Config {
    type Error = color_eyre::Report;

    fn try_from(path: &PathBuf) -> Result<Self> {
        let data = fs::read_to_string(path)?;
        toml::from_str(&data).map_err(Into::into)
    }
}
