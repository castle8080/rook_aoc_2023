use std::path::Path;

use lazy_static::lazy_static;
use regex::Regex;

use crate::aocbase::{AOCError, AOCResult};
use crate::aocio::process_lines;

lazy_static! {
    static ref GAME_REGEX: Regex = Regex::new(r"^Game (\d+): (.*)").unwrap();
    static ref COLOR_COUNT_REGEX: Regex = Regex::new(r"^\s*(\d+)\s+(red|green|blue)").unwrap();
}

#[derive(Debug)]
pub struct CubeCounts {
    pub red: i32,
    pub green: i32,
    pub blue: i32,
}

impl CubeCounts {

    pub fn default() -> CubeCounts {
        CubeCounts { red: 0, green: 0, blue: 0 }
    }

    pub fn power_set(&self) -> i32 {
        self.red * self.green * self.blue
    }

    pub fn parse(input: &str) -> AOCResult<CubeCounts> {
        let mut cube_counts = CubeCounts::default();

        for c_count_str in input.split(',') {

            let c_count_cap = COLOR_COUNT_REGEX
                .captures(c_count_str)
                .ok_or_else(|| AOCError::ParseError(format!("Invalid color count: {}", c_count_str)))?;

            let c_count = c_count_cap
                .get(1)
                .ok_or_else(|| AOCError::InvalidRegexOperation("incorrect capture".into()))?
                .as_str()
                .parse::<i32>()?;

            let color = c_count_cap
                .get(2)
                .ok_or_else(|| AOCError::InvalidRegexOperation("incorrect capture".into()))?
                .as_str();

            match color {
                "red" => cube_counts.red = c_count,
                "green" => cube_counts.green = c_count,
                "blue" => cube_counts.blue = c_count,
                _ => {
                    return Err(AOCError::ParseError(format!("Invalid color: {}", color)));
                }
            }
        }

        Ok(cube_counts)
    }
}

#[derive(Debug)]
pub struct CubeCountGame {
    pub id: i32,
    pub count_sets: Vec<CubeCounts>,
}

impl CubeCountGame {

    pub fn are_total_counts_possible(&self, possible: &CubeCounts) -> bool {
        let max_counts = self.get_max_counts();
        possible.red >= max_counts.red &&
            possible.green >= max_counts.green &&
            possible.blue >= max_counts.blue
    }

    pub fn get_max_counts(&self) -> CubeCounts {
        let red = self.count_sets.iter().map(|c| c.red).max().unwrap_or(0);
        let green = self.count_sets.iter().map(|c| c.green).max().unwrap_or(0);
        let blue = self.count_sets.iter().map(|c| c.blue).max().unwrap_or(0);

        CubeCounts { red, green, blue }
    }

    pub fn parse(input: impl AsRef<str>) -> AOCResult<CubeCountGame> {
        let game_cap = GAME_REGEX
            .captures(input.as_ref())
            .ok_or_else(|| AOCError::ParseError(format!("Invalid game: {}", input.as_ref())))?;

        let id = game_cap
            .get(1)
            .ok_or_else(|| AOCError::InvalidRegexOperation("incorect capture".into()))?
            .as_str()
            .parse::<i32>()?;
        
        let count_sets = game_cap
            .get(2)
            .ok_or_else(|| AOCError::InvalidRegexOperation("incorect capture".into()))?
            .as_str()
            .split(';')
            .into_iter()
            .map(CubeCounts::parse)
            .collect::<AOCResult<Vec<CubeCounts>>>()?;

        Ok(CubeCountGame { id, count_sets })
    }
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let possible_counts = CubeCounts {
        red: 12,
        green: 13,
        blue: 14,
    };

    let mut result = 0;

    process_lines(input, |line| {
        let game = CubeCountGame::parse(line)?;
        if game.are_total_counts_possible(&possible_counts) {
            result += game.id;
        }
        Ok(())
    })?;

    Ok(result.to_string())
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    let mut result = 0;

    process_lines(input, |line| {
        let game = CubeCountGame::parse(line)?;
        result += game.get_max_counts().power_set();
        Ok(())
    })?;

    Ok(result.to_string())
}