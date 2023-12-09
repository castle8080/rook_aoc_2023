use std::num::ParseIntError;
use std::path::Path;

use crate::aocbase::{AOCError, AOCResult};
use crate::aocio::each_line;

#[derive(Debug)]
pub struct RaceWinner {
    pub time: i64,
    pub distance: i64,
}

impl RaceWinner {

    pub fn get_n_ways_to_beat(&self) -> i64 {
        let (winner_h_left, winner_h_right) = self.calculate_hold_times();

        let win_start = (winner_h_left as i64) + 1;
        let win_end = winner_h_right as i64;
    
        if win_end > win_start {
            win_end - win_start + 1
        }
        else {
            0
        }
    }

    pub fn calculate_hold_times(&self) -> (f64, f64) {
        // d = -hold_time**2 + total_time * hold_time
        // 0 = -hold_time**2 + total_time * hold_time - d
    
        // Use quadratic formula to get hold times.
    
        let a: f64 = -1.0;
        let b: f64 = self.time as f64;
        let c: f64 = -self.distance as f64;
    
        let s_dicriminant = (b * b - 4.0 * a * c).sqrt();
        let x_1 = (-b - s_dicriminant) / (2.0 * a);
        let x_2 = (-b + s_dicriminant) / (2.0 * a);
    
        if x_1 < x_2 {
            (x_1, x_2)
        }
        else {
            (x_2, x_1)
        }
    }
}

#[derive(Debug)]
pub struct RaceRecords {
    winners: Vec<RaceWinner>,
}

impl RaceRecords {
    pub fn parse(input: impl AsRef<Path>) -> AOCResult<Self> {
        RaceRecords::_parse(input, |line| line.into())
    }

    pub fn parse_bad_kearning(input: impl AsRef<Path>) -> AOCResult<Self> {
        RaceRecords::_parse(input, |line| line.trim().replace(' ', "").into())
    }

    fn _parse<F>(input: impl AsRef<Path>, line_xform: F) -> AOCResult<Self>
        where F: Fn(&String) -> String
    {
        let mut time_numbers: Option<Vec<i64>> = None;
        let mut distance_numbers: Option<Vec<i64>> = None;

        each_line(input, |line| {
            if line.starts_with("Time:") {
                time_numbers = Some(parse_info_numbers(line_xform(line))?);
            }
            else if line.starts_with("Distance:") {
                distance_numbers = Some(parse_info_numbers(line_xform(line))?);
            }
            Ok(())
        })?;

        match (time_numbers, distance_numbers) {
            (Some(tn), Some(dn)) => {
                if tn.len() != dn.len() {
                    return Err(AOCError::ParseError(format!("Mismatched time and distance.")));
                }
                let winners: Vec<RaceWinner> = tn
                    .iter()
                    .zip(&dn)
                    .map(|(time, distance)| RaceWinner { time: *time, distance: *distance })
                    .collect();

                Ok(RaceRecords { winners })
            },
            _ => {
                Err(AOCError::ParseError("Invalid race winners data.".into()))
            }
        }
    }
}

fn parse_info_numbers(line: impl AsRef<str>) -> AOCResult<Vec<i64>> {
    Ok(line
        .as_ref()
        .split(':')
        .nth(1)
        .ok_or_else(|| AOCError::ParseError(format!("Invalid line: {}", line.as_ref())))?
        .split_ascii_whitespace()
        .filter(|s| s.len() > 0)
        .map(|s| s.parse::<i64>())
        .collect::<Result<Vec<i64>, ParseIntError>>()?)
}

fn run_part(race_records: &RaceRecords) -> AOCResult<String> {
    let mut result = 1;

    for race_record in &race_records.winners {
        let ways_to_win = race_record.get_n_ways_to_beat();
        result *= ways_to_win;
    }

    Ok(result.to_string())
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let race_records = RaceRecords::parse(input)?;
    run_part(&race_records)
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    let race_records = RaceRecords::parse_bad_kearning(input)?;
    run_part(&race_records)
}