use std::io::{BufRead, BufReader, Write};
use unicode_normalization::{is_nfc_stream_safe_quick, IsNormalized, UnicodeNormalization};
use utf8_io::Utf8Reader;

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let lock = BufReader::new(Utf8Reader::new(stdin.lock()));
    let mut out = stdout.lock();
    for line in lock.lines() {
        let mut line = line?;
        line.push('\n');
        if is_nfc_stream_safe_quick(line.chars()) == IsNormalized::Yes {
            out.write_all(line.as_bytes())?;
        } else {
            out.write_all(
                line.chars()
                    .stream_safe()
                    .nfc()
                    .collect::<String>()
                    .as_bytes(),
            )?;
        }
    }
    Ok(())
}
