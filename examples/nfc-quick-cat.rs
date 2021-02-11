use std::io::{BufRead, Write};
use unicode_normalization::{is_nfc_quick, IsNormalized, UnicodeNormalization};

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let lock = stdin.lock();
    let mut out = stdout.lock();
    for line in lock.lines() {
        let mut line = line?;
        line.push('\n');
        if is_nfc_quick(line.chars()) == IsNormalized::Yes {
            out.write_all(line.as_bytes())?;
        } else {
            out.write_all(line.chars().nfc().collect::<String>().as_bytes())?;
        }
    }
    Ok(())
}
