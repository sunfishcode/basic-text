use std::io::Write;
use utf8_io::{Utf8Reader, Utf8Writer, copy_str};

fn main() -> anyhow::Result<()> {
    let mut reader = Utf8Reader::new(std::io::stdin());
    let mut writer = Utf8Writer::new(std::io::stdout());
    copy_str(&mut reader, &mut writer)?;
    writer.flush()?;
    Ok(())
}
