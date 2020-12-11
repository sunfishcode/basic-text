use io_ext::{ReadExt, WriteExt};
use io_ext_adapters::{ExtReader, ExtWriter};
use std::io::Write;

fn main() -> anyhow::Result<()> {
    let mut reader = ExtReader::new(std::io::stdin());
    let mut stdout = ExtWriter::new(std::io::stdout());
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
