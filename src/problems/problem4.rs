use std::collections::HashSet;
use std::cmp::min;
use std::num::ParseIntError;
use std::path::Path;

use lazy_static::lazy_static;
use regex::Regex;

use crate::aocbase::AOCResult;
use crate::aocio::each_line;
use crate::regex_ext::{RegexExt, CapturesExt};

lazy_static! {
    static ref CARD_REGEX: Regex = Regex::new(r"Card +(\d+):([ \d]*)\|([ \d]*)").unwrap();
}

#[derive(Debug)]
#[allow(dead_code)]
struct GameCard {
    id: i32,
    winning_numbers: HashSet<i32>,
    numbers: HashSet<i32>,
}

impl GameCard {

    fn to_hashset(num_list: &str) -> AOCResult<HashSet<i32>> {
        Ok(num_list
            .split(" ")
            .filter(|s| s.len() > 0)
            .map(|s| s.parse::<i32>())
            .collect::<Result<HashSet<i32>, ParseIntError>>()?)
    }

    //Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
    pub fn parse(line: impl AsRef<str>) -> AOCResult<GameCard> {
        let line = line.as_ref();

        let cap = CARD_REGEX.captures_must(line)?;

        let id = cap.get_group(1)?.parse::<i32>()?;
        let winning_numbers: HashSet<i32> = Self::to_hashset(cap.get_group(2)?)?;
        let numbers: HashSet<i32> = Self::to_hashset(cap.get_group(3)?)?;

        Ok(GameCard { id, winning_numbers, numbers })
    }

    pub fn match_count(&self) -> usize {
        self.numbers.iter()
            .filter(|n| self.winning_numbers.contains(n))
            .count()
    }

    pub fn score(&self) -> usize {
        let count = self.match_count();

        if count == 0 { 0 } else { (2 as usize).pow((count - 1) as u32) }
    }
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let mut result = 0;

    each_line(input, |line| {
        let card = GameCard::parse(line)?;
        result += card.score();
        Ok(())
    })?;

    Ok(result.to_string())
}


pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    let mut games: Vec<GameCard> = Vec::new();
    each_line(input, |line| {
        games.push(GameCard::parse(line)?);
        Ok(())
    })?;

    // Count the extra cards you get.
    let mut card_counts = vec![1 as usize; games.len()];

    // Apply wins to get new cards.
    for idx in 0..card_counts.len() {
        let match_count = games[idx].match_count();
        let cur_card_count = card_counts[idx];

        let w_start = idx + 1;
        let w_end = min(idx + 1 + match_count, card_counts.len());

        for w_idx in w_start .. w_end {
            card_counts[w_idx] += cur_card_count;
        }
    }

    let result: usize = card_counts.iter().sum();

    Ok(result.to_string())
}