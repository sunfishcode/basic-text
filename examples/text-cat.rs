use io_ext::WriteExt;
use io_ext_adapters::{ExtReader, ExtWriter};
use textual::{copy_text, TextReader, TextWriter};

fn main() -> anyhow::Result<()> {
    let mut reader = TextReader::new(ExtReader::new(std::io::stdin()));
    let mut writer = TextWriter::new(ExtWriter::new(std::io::stdout()));
    copy_text(&mut reader, &mut writer)?;
    writer.close()?;
    Ok(())
}
