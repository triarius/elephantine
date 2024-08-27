use color_eyre::Result;

fn main() -> Result<()> {
    let input = std::io::BufReader::new(std::io::stdin());
    let mut output = std::io::stdout();
    elephantine::pinentry::listen(input, &mut output)
}
