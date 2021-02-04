use std::io::{BufRead, Write};
use unicode_normalization::UnicodeNormalization;

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let lock = stdin.lock();
    let mut out = stdout.lock();
    for line in lock.lines() {
        let mut line = line?;
        line.push('\n');
        out.write_all(line.chars().nfc().collect::<String>().as_bytes())?;
    }
    Ok(())
}
