use std::io::{copy, Write};

fn main() -> anyhow::Result<()> {
    let mut reader = std::io::stdin();
    let mut writer = std::io::stdout();
    copy(&mut reader, &mut writer)?;
    writer.flush()?;
    Ok(())
}
