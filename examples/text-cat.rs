use basic_text::{copy_text, TextReader, TextWriter};
use layered_io::WriteLayered;

fn main() -> anyhow::Result<()> {
    let mut reader = TextReader::new(std::io::stdin());
    let mut writer = TextWriter::new(std::io::stdout());
    copy_text(&mut reader, &mut writer)?;
    writer.close()?;
    Ok(())
}
