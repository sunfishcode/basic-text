use io_ext::WriteExt;
use io_ext_adapters::{ExtReader, ExtWriter};
use textual::{copy_str, Utf8Reader, Utf8Writer};

fn main() -> anyhow::Result<()> {
    let mut reader = Utf8Reader::new(ExtReader::new(std::io::stdin()));
    let mut writer = Utf8Writer::new(ExtWriter::new(std::io::stdout()));
    copy_str(&mut reader, &mut writer)?;
    writer.close()?;
    Ok(())
}
