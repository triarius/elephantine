use clap_serde_derive::{clap::Parser, ClapSerde};
use color_eyre::Result;
use elephantine::config::Config;
use elephantine::Listener;
use std::{
    io::{stdin, stdout, BufReader},
    path::PathBuf,
};

/// Implements the pinentry protocol and uses a configurable frontend for PIN input.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The debug level.
    #[arg(short, long, env = "ELEPHANTINE_DEBUG", action = clap::ArgAction::Count)]
    debug: u8,

    /// Path to the configuration file.
    #[arg(long, env = "ELEPHANTINE_CONFIG_FILE", value_name = "FILE", default_value = default_config_file())]
    config_file: PathBuf,

    /// The configuration options.
    #[command(flatten)]
    pub config: <Config as ClapSerde>::Opt,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let config = if args.config_file.exists() {
        Config::try_from(&args.config_file)?
    } else {
        Config::from(args.config)
    };

    let input = BufReader::new(stdin());
    let mut output = stdout();
    Listener::new(config).listen(input, &mut output)
}

fn default_config_file() -> String {
    directories::ProjectDirs::from("org", "elephantine", "elephantine").map_or_else(
        || "elephantine.toml".to_string(),
        |dirs| {
            dirs.config_dir()
                .join("elephantine.toml")
                .to_string_lossy()
                .to_string()
        },
    )
}
