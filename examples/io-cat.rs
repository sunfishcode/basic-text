use io_ext::{ReadExt, WriteExt};
use io_ext_adapters::{StdReader, StdWriter};
use std::io::Write;

fn main() -> anyhow::Result<()> {
    let mut reader = StdReader::new(std::io::stdin());
    let mut stdout = StdWriter::new(std::io::stdout());
    let mut buf = [0; 8];
    loop {
        let size_and_status = reader.read_with_status(&mut buf)?;
        stdout.write_all(&buf[..size_and_status.0])?;
        stdout.flush_with_status(size_and_status.1)?;
        if size_and_status.1.is_end() {
            return Ok(());
        }
    }
}
