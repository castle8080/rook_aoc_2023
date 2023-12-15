use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

use crate::aocbase::{AOCResult, AOCError};

fn get_strings(input: impl AsRef<Path>) -> AOCResult<Vec<String>> {
    let reader = BufReader::new(File::open(input)?);
    Ok(reader
        .lines()
        .nth(0)
        .ok_or_else(|| AOCError::ParseError("Expected a line.".into()))??
        .trim()
        .split(',')
        .map(|s| s.to_string())
        .collect::<Vec<String>>())
}

/*
    Determine the ASCII code for the current character of the string.
    Increase the current value by the ASCII code you just determined.
    Set the current value to itself multiplied by 17.
    Set the current value to the remainder of dividing itself by 256.
*/
pub fn string_hash(input: impl AsRef<str>) -> i64 {
    input
        .as_ref()
        .as_bytes()
        .iter()
        .fold(0 as i64, |current, b| ((current + *b as i64) * 17) % 256)
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    Ok(get_strings(input)?
        .iter()
        .map(string_hash)
        .sum::<i64>()
        .to_string())
}
