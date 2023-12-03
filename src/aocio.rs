
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