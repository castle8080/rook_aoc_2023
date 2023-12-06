use std::num::ParseIntError;
use std::path::Path;

use crate::aocbase::{AOCError, AOCResult};
use crate::aocio::process_lines;

#[derive(Debug)]
pub struct RaceWinner {
    pub time: i64,
    pub distance: i64,
}

impl RaceWinner {
    pub fn parse(input: impl AsRef<Path>) -> AOCResult<Self> {
        let mut time: Option<i64> = None;
        let mut distance: Option<i64> = None;

        process_lines(input, |line| {
            if line.starts_with("Time:") {
                time = Some(parse_info_numbers_bad_kearning(line)?);
            }
            else if line.starts_with("Distance:") {
                distance = Some(parse_info_numbers_bad_kearning(line)?);
            }
            Ok(())
        })?;

        match (time, distance) {
            (Some(time), Some(distance)) => Ok(RaceWinner { time, distance }),
            _ => Err(AOCError::ParseError(format!("Missing race information.")))
        }
    }
}

#[derive(Debug)]
pub struct RaceRecords {
    winners: Vec<RaceWinner>,
}

impl RaceRecords {
    pub fn parse(input: impl AsRef<Path>) -> AOCResult<Self> {
        let mut time_numbers: Option<Vec<i64>> = None;
        let mut distance_numbers: Option<Vec<i64>> = None;

        process_lines(input, |line| {
            if line.starts_with("Time:") {
                time_numbers = Some(parse_info_numbers(line)?);
            }
            else if line.starts_with("Distance:") {
                distance_numbers = Some(parse_info_numbers(line)?);
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

fn parse_info_numbers_bad_kearning(line: &String) -> AOCResult<i64> {
    Ok(line
        .split(':')
        .nth(1)
        .ok_or_else(|| AOCError::ParseError(format!("Invalid line: {line}")))?
        .trim()
        .replace(" ", "")
        .parse::<i64>()?)
}

fn parse_info_numbers(line: &String) -> AOCResult<Vec<i64>> {
    Ok(line
        .split(':')
        .nth(1)
        .ok_or_else(|| AOCError::ParseError(format!("Invalid line: {line}")))?
        .split_ascii_whitespace()
        .filter(|s| s.len() > 0)
        .map(|s| s.parse::<i64>())
        .collect::<Result<Vec<i64>, ParseIntError>>()?)
}

fn get_distance(total_time: i64, hold_time: i64) -> i64 {
    hold_time * (total_time - hold_time)
}

fn get_hold_times(total_time: i64, distance: i64) -> (f64, f64) {
    // d = -hold_time**2 + total_time * hold_time
    // 0 = -hold_time**2 + total_time * hold_time - d

    // Use quadratic formula to get hold times.

    let a: f64 = -1.0;
    let b: f64 = total_time as f64;
    let c: f64 = -distance as f64;

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

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let race_records = RaceRecords::parse(input)?;
    let mut result = 1;

    for race_record in &race_records.winners {
        let mut ways_to_win = 0;

        for hold_time in 1..race_record.time {
            let d = get_distance(race_record.time, hold_time);
            if d > race_record.distance {
                ways_to_win += 1;
            }
        }

        result *= ways_to_win;
    }

    Ok(result.to_string())
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    let winner = RaceWinner::parse(input)?;
    let mut result = 0;

    let (winner_h_left, winner_h_right) = get_hold_times(winner.time, winner.distance);

    let win_start = (winner_h_left as i64) + 1;
    let win_end = winner_h_right as i64;

    if win_end > win_start {
        result = win_end - win_start + 1;
    }

    Ok(result.to_string())
}