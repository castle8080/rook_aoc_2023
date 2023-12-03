
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

use crate::aocbase::AOCResult;

pub fn process_lines<F>(input: impl AsRef<Path>, mut f: F) -> AOCResult<()>
    where F: FnMut(&String) -> AOCResult<()>
{
    let mut reader = BufReader::new(File::open(input)?);
    let mut buffer = String::new();

    while reader.read_line(&mut buffer)? > 0 {
        f(&buffer)?;
        buffer.clear();
    }

    Ok(())
}

pub fn read_lines_as_bytes(input: impl AsRef<Path>) -> AOCResult<Vec<Vec<u8>>> {
    let mut reader = BufReader::new(File::open(input)?);
    let mut buffer: Vec<u8> = Vec::new();
    buffer.reserve(1024);

    let mut results: Vec<Vec<u8>> = Vec::new();

    while reader.read_until(b'\n', &mut buffer)? > 0 {
        buffer.pop();
        results.push(buffer.clone());
        buffer.clear();
    }

    Ok(results)
}