use std::io::{copy, stdin, stdout, Write};
use utf8_io::Utf8Reader;

fn main() -> anyhow::Result<()> {
    let mut reader = Utf8Reader::new(stdin());
    let mut writer = stdout();
    copy(&mut reader, &mut writer)?;
    writer.flush()?;
    Ok(())
}
