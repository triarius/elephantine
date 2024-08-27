pub mod pinentry;
pub mod request;
pub mod response;

pub mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use color_eyre::Result;
use pinentry::{listen, walker_get_pin};
use std::io::{stdin, stdout, BufReader};

fn main() -> Result<()> {
    let input = BufReader::new(stdin());
    let mut output = stdout();
    listen(input, &mut output, walker_get_pin)?;

    Ok(())
}
