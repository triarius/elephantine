pub mod pinentry;
pub mod request;
pub mod response;

pub mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use color_eyre::Result;
use pinentry::{listen, walker_get_pin};
use std::{
    env,
    fs::OpenOptions,
    io::{stdin, stdout, BufReader},
};

fn main() -> Result<()> {
    let home = env::var("HOME")?;
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{home}/elephantine.log"))?;

    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .init();

    let input = BufReader::new(stdin());
    let mut output = stdout();
    listen(input, &mut output, walker_get_pin)?;

    Ok(())
}
