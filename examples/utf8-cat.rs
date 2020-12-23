use io_ext::{ReadExt, WriteExt};
use io_ext_adapters::{ExtReader, ExtWriter};
use std::io::Write;
use text_formats::{Utf8Reader, Utf8Writer};

fn main() -> anyhow::Result<()> {
    let mut reader = Utf8Reader::new(ExtReader::new(std::io::stdin()));
    let mut stdout = Utf8Writer::new(ExtWriter::new(std::io::stdout()));
    let mut buf = [0; 8];
    loop {
        let (size, status) = reader.read_with_status(&mut buf)?;
        stdout.write_all(&buf[..size])?;
        stdout.flush_with_status(status)?;
        if status.is_end() {
            return Ok(());
        }
    }
}
