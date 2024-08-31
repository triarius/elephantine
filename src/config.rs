use clap_serde_derive::ClapSerde;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, time::Duration};

#[allow(clippy::module_name_repetitions)]
#[derive(ClapSerde, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Config {
    /// Timeout in seconds for requests that show dialogs to the user.
    /// E.g. GETPIN, CONFIRM, etc.
    #[arg(short, long, value_name = "TIMEOUT_IN_SECONDS", value_parser = parse_duration, default_value = "300")]
    pub timeout_in_seconds: Option<Duration>,

    /// The command to run when a user input is required.
    /// It must print the input to stdout.
    #[arg(
        short,
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
