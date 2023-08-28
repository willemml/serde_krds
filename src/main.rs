#![feature(buf_read_has_data_left)]
#![feature(iter_next_chunk)]

use std::io::Write;

mod cli;
pub mod de;
pub mod error;
pub mod ser;
pub mod file_formats;

pub(crate) const MAGIC: &[u8; 8] = b"\x00\x00\x00\x00\x00\x1A\xB1\x26";

fn main() -> Result<(), std::io::Error> {
    let yjr = file_formats::example_files::yjr_file_1();

    let serialized = ser::to_bytevec(&yjr).unwrap();

    let mut stdout = std::io::stdout();
    stdout.write_all(&serialized)?;
    stdout.flush()?;
    Ok(())
}
