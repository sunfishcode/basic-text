use io_ext::{ReadExt, WriteExt};
use io_ext_adapters::{StdReader, StdWriter};
use std::io::Write;
use text_streams::{TextReader, TextWriter, NORMALIZATION_BUFFER_SIZE};

fn main() -> anyhow::Result<()> {
    let mut reader = TextReader::new(StdReader::new(std::io::stdin()));
    let mut stdout = TextWriter::new(StdWriter::new(std::io::stdout()));
    let mut buf = [0; NORMALIZATION_BUFFER_SIZE];
    loop {
        let (size, status) = reader.read_with_status(&mut buf)?;
        stdout.write_all(&buf[..size])?;
        stdout.flush_with_status(status)?;
        if status.is_end() {
            return Ok(());
        }
    }
}
