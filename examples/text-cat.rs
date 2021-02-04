use layered_io::{LayeredReader, LayeredWriter, WriteLayered};
use basic_text::{copy_text, TextReader, TextWriter};
use utf8_io::{Utf8Reader, Utf8Writer};

fn main() -> anyhow::Result<()> {
    let mut reader = TextReader::new(Utf8Reader::new(LayeredReader::new(std::io::stdin())));
    let mut writer = TextWriter::new(Utf8Writer::new(LayeredWriter::new(std::io::stdout())));
    copy_text(&mut reader, &mut writer)?;
    writer.close()?;
    Ok(())
}
