use std::io::{copy, stdin, stdout, Write};

fn main() -> anyhow::Result<()> {
    let mut reader = stdin();
    let mut writer = stdout();
    copy(&mut reader, &mut writer)?;
    writer.flush()?;
    Ok(())
}
