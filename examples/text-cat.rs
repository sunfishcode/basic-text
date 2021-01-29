use layered_io::{LayeredReader, LayeredWriter, WriteLayered};
use text_str::{copy_text, TextReader, TextWriter};

fn main() -> anyhow::Result<()> {
    let mut reader = TextReader::new(LayeredReader::new(std::io::stdin()));
    let mut writer = TextWriter::new(LayeredWriter::new(std::io::stdout()));
    copy_text(&mut reader, &mut writer)?;
    writer.close()?;
    Ok(())
}
