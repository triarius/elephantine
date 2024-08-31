use clap_serde_derive::{clap::Parser, ClapSerde};
use color_eyre::Result;
use elephantine::config::Config;
use elephantine::Listener;
use std::{
    io::{stdin, stdout, BufReader},
    path::PathBuf,
};

/// Implements the pinentry protocol and uses walker for PIN input.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The debug level.
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// Path to the configuration file.
    #[arg(long, value_name = "FILE", default_value = "config.toml")]
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
