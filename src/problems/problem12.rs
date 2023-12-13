use std::path::Path;
use std::num::ParseIntError;
use std::collections::HashMap;

use crate::aocbase::{AOCResult, AOCError};
use crate::aocio::each_line;

#[derive(Debug, Copy, Clone)]
pub enum SpringCondition {
    Operational = 0,
    Damaged,
    Unknown,
}

impl SpringCondition {
    pub fn parse(c: char) -> AOCResult<SpringCondition> {
        use SpringCondition::*;
        Ok(match c {
            '.' => Operational,
            '#' => Damaged,
            '?' => Unknown,
            _ => {
                return Err(AOCError::ParseError(format!("Invalid SpringCondition: {}", c)));
            }
        })
    }
}

#[derive(Debug)]
pub struct SpringsCondition {
    pub springs: Vec<SpringCondition>,
    pub damaged_sequences: Vec<u32>,
}

impl SpringsCondition {

    pub fn expand(&self, amount: u32) -> SpringsCondition {
        let mut springs = self.springs.clone();
        let mut damaged_sequences = self.damaged_sequences.clone();

        for _ in 0..amount {
            springs.push(SpringCondition::Unknown);
            for s in &self.springs {
                springs.push(*s);
            }

            for ds in &self.damaged_sequences {
                damaged_sequences.push(*ds);
            }
        }

        SpringsCondition { springs, damaged_sequences }
    }

    pub fn parse(line: impl AsRef<str>) -> AOCResult<SpringsCondition> {
        let parts: Vec<&str> = line.as_ref().trim().split_ascii_whitespace().collect();

        if parts.len() != 2 {
            return Err(AOCError::ParseError(format!("Invalid SpringsCondition: {}", line.as_ref())));
        }

        let springs = parts[0]
            .chars()
            .map(|c| SpringCondition::parse(c))
            .collect::<AOCResult<Vec<SpringCondition>>>()?;

        let damaged_sequences = parts[1]
            .split(',')
            .map(|s| s.trim())
            .filter(|s| s.len() > 0)
            .map(|s| s.parse::<u32>())
            .collect::<Result<Vec<u32>, ParseIntError>>()?;

        Ok(SpringsCondition { springs, damaged_sequences })
    }
}

pub struct SpringsConditionsSolver<'a> {
    pub springs_condition: &'a SpringsCondition,
    pub match_count_cache: HashMap<(usize, usize), u64>,
}

impl<'a> SpringsConditionsSolver<'a> {
    
    pub fn new(springs_condition: &'a SpringsCondition) -> Self {
        Self { springs_condition, match_count_cache: HashMap::new() }
    }

    pub fn solve(&mut self) -> u64 {
        self.match_count_cache = HashMap::new();
        self.search_for_matches(0, 0)
    }
    
    fn set_match_count(&mut self, pos: usize, ds_pos: usize, match_count: u64) -> u64 {
        self.match_count_cache.insert((pos, ds_pos), match_count);
        match_count
    }

    fn is_match(&self, pos: usize, ds_pos: usize) -> bool {
        pos >= self.springs_condition.springs.len() && ds_pos >= self.springs_condition.damaged_sequences.len()
    }

    fn search_for_matches(&mut self, pos: usize, ds_pos: usize)
        -> u64
    {
        // Check for value in cache
        if let Some(_match_count) = self.match_count_cache.get(&(pos, ds_pos)) {
            return *_match_count;
        }

        let springs = &self.springs_condition.springs;
        let damaged_sequences = &self.springs_condition.damaged_sequences;

        // This is a match
        if self.is_match(pos, ds_pos) {
            return self.set_match_count(pos, ds_pos, 1);
        }

        // At the end with no match
        if pos >= springs.len() {
            return self.set_match_count(pos, ds_pos, 0);
        }

        let mut match_count: u64 = 0;

        // Treat current pos as operational
        match springs[pos] {
            SpringCondition::Operational|SpringCondition::Unknown => {
                match_count += self.search_for_matches(pos+1, ds_pos);
            },
            _ => {}
        }

        // Treat current pos as damaged
        match springs[pos] {
            // Try consuming next sequence.
            SpringCondition::Damaged|SpringCondition::Unknown => {
                if ds_pos >= damaged_sequences.len() {
                    return self.set_match_count(pos, ds_pos, match_count);
                }
                let ds_len = damaged_sequences[ds_pos] as usize;
                if pos + ds_len as usize > springs.len() {
                    // Not enough stuff for the damaged sequence
                    return self.set_match_count(pos, ds_pos, match_count);
                }
                
                // Make sure there are no operational ones for this sequence.
                for i in 0..ds_len {
                    if let SpringCondition::Operational = springs[pos + i] {
                        return self.set_match_count(pos, ds_pos, match_count);
                    }
                }

                // Peek ahead to make sure damaged sequence doesn't continue.
                let new_pos = pos + ds_len;
                match springs.get(new_pos) {
                    None => {
                        match_count += self.search_for_matches(new_pos, ds_pos+1);
                        return self.set_match_count(pos, ds_pos, match_count);
                    },
                    Some(SpringCondition::Operational|SpringCondition::Unknown) => {
                        // skip next as it must be treated as opertional
                        match_count += self.search_for_matches(new_pos+1, ds_pos+1);
                        return self.set_match_count(pos, ds_pos, match_count);
                    },
                    _ => {
                        return self.set_match_count(pos, ds_pos, match_count);
                    }
                }
            },
            _ => {
                return self.set_match_count(pos, ds_pos, match_count);
            }
        }
    }

}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let mut total: u64 = 0;

    each_line(input, |line| {
        let springs_condition = SpringsCondition::parse(line)?;
        let mut solver = SpringsConditionsSolver::new(&springs_condition);
        total += solver.solve();
        Ok(())
    })?;

    Ok(total.to_string())
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    let mut total: u64 = 0;

    each_line(input, |line| {
        let springs_condition = SpringsCondition::parse(line)?;
        let x_springs_condition = springs_condition.expand(4);
        let mut solver = SpringsConditionsSolver::new(&x_springs_condition);
        total += solver.solve();
        Ok(())
    })?;

    Ok(total.to_string())
}
