use std::path::Path;
use std::collections::HashMap;

use regex::bytes::Regex as BRegex;

use crate::aocbase::AOCResult;
use crate::aocio::read_lines_as_bytes;

fn parse_i32(input: &[u8]) -> AOCResult<i32> {
    let mut n: i32 = 0;
    for c in input {
        n = (n * 10) + (c - b'0') as i32;
    }
    Ok(n)
}

fn find_adjacent<F>(data: &Vec<Vec<u8>>, row: usize, start: usize, end: usize, f: F) -> Vec<(usize, usize)>
    where F: Fn(u8) -> bool
{
    let row_start = if row > 0 { row - 1 } else { row };
    let row_end = if row + 1 < data.len() { row + 1 } else { row };

    let mut adjacent_locations: Vec<(usize, usize)> = Vec::new();

    for check_row in row_start ..= row_end {
        let data_row = &data[check_row];
        let col_start = if start > 0 { start - 1 } else { start };
        let col_end = if end < data_row.len() { end } else { data_row.len() - 1 };

        if col_start < data_row.len() {
            for check_col in col_start ..= col_end {
                if f(data_row[check_col]) {
                    adjacent_locations.push((check_row, check_col));
                }
            }
        }
    }
    adjacent_locations
}

fn is_adjacent<F>(data: &Vec<Vec<u8>>, row: usize, start: usize, end: usize, f: F) -> bool
    where F: Fn(u8) -> bool
{
    find_adjacent(data, row, start, end, f).len() > 0
}

fn is_symbol(b: u8) -> bool {
    !(b >= b'0' && b <= b'9') && b != b'.'
}

fn is_gear(b: u8) -> bool {
    b == b'*'
}

fn is_symbol_adjacent(data: &Vec<Vec<u8>>, row: usize, start: usize, end: usize) -> bool {
    is_adjacent(data, row, start, end, is_symbol)
}

fn find_adjacent_gears(data: &Vec<Vec<u8>>, row: usize, start: usize, end: usize) -> Vec<(usize, usize)> {
    find_adjacent(data, row, start, end, is_gear)
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let data = read_lines_as_bytes(input)?;
    let num_regex = BRegex::new(r"(\d+)")?;
    let mut result = 0;

    for (row, line) in data.iter().enumerate() {
        for m in num_regex.find_iter(&line) {
            let is_part_num = is_symbol_adjacent(&data, row, m.start(), m.end());
            if is_part_num {
                result += parse_i32(m.as_bytes())?;
            }
        }
    }

    Ok(result.to_string())
}

fn read_gear_map(input: impl AsRef<Path>) -> AOCResult<HashMap<(usize, usize), Vec<i32>>> {
    let data = read_lines_as_bytes(input)?;
    let num_regex = BRegex::new(r"(\d+)")?;

    let mut gear_map: HashMap<(usize, usize), Vec<i32>> = HashMap::new();

    for (row, line) in data.iter().enumerate() {
        for m in num_regex.find_iter(&line) {
            let n = parse_i32(m.as_bytes())?;
            let adjacent_gears = find_adjacent_gears(&data, row, m.start(), m.end());

            for gear_loc in adjacent_gears {
                match gear_map.get_mut(&gear_loc) {
                    None => {
                        gear_map.insert(gear_loc, vec![n]);
                    },
                    Some(adjacent_nums) => {
                        adjacent_nums.push(n);
                    }
                }
            }
        }
    }

    Ok(gear_map)
}

fn calculate_gear_ratio_sums(gear_map: &HashMap<(usize, usize), Vec<i32>>) -> i32 {
    let mut result = 0;

    for (_gear_loc, adjacent_nums) in gear_map {
        if adjacent_nums.len() == 2 {
            result += adjacent_nums[0] * adjacent_nums[1];
        }
    }

    result
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    let gear_map = read_gear_map(input)?;
    let result = calculate_gear_ratio_sums(&gear_map);

    Ok(result.to_string())
}