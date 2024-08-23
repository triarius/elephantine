use color_eyre::Result;

fn main() -> Result<()> {
    let writer = &mut std::io::stdout();
    send(writer, "Hello, world!")?;
    Ok(())
}

fn send(writer: &mut impl std::io::Write, msg: &str) -> Result<()> {
    writeln!(writer, "{msg}")?;
    writer.flush()?;
    Ok(())
}
