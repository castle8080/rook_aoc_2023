use std::path::Path;
use std::num::ParseIntError;

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

    pub fn find_matching_unknown_solutions(&self) -> u32 {
        let mut count: u32 = 0;
        self.search_for_matches(0, 0, &mut count);
        count
    }
    
    fn is_match(&self, pos: usize, ds_pos: usize) -> bool {
        pos >= self.springs.len() && ds_pos >= self.damaged_sequences.len()
    }

    fn search_for_matches(&self, pos: usize, ds_pos: usize, count: &mut u32)
    {
        // This is a match
        if self.is_match(pos, ds_pos) {
            *count = *count + 1;
        }

        // At the end with no match
        if pos >= self.springs.len() {
            return;
        }

        // Treat current pos as operational
        match self.springs[pos] {
            SpringCondition::Operational|SpringCondition::Unknown => {
                self.search_for_matches(pos+1, ds_pos, count);
            },
            _ => {}
        }

        // Treat current pos as damaged
        match self.springs[pos] {
            // Try consuming next sequence.
            SpringCondition::Damaged|SpringCondition::Unknown => {
                if ds_pos >= self.damaged_sequences.len() {
                    return;
                }
                let ds_len = self.damaged_sequences[ds_pos] as usize;
                if pos + ds_len as usize > self.springs.len() {
                    // Not enough stuff for the damaged sequence
                    return;
                }
                
                // Make sure there are no operational ones for this sequence.
                for i in 0..ds_len {
                    if let SpringCondition::Operational = self.springs[pos + i] {
                        return;
                    }
                }

                // Peek ahead to make sure damaged sequence doesn't continue.
                let new_pos = pos + ds_len;
                match self.springs.get(new_pos) {
                    None => {
                        self.search_for_matches(new_pos, ds_pos+1, count);
                    },
                    Some(SpringCondition::Operational|SpringCondition::Unknown) => {
                        // skip next as it must be treated as opertional
                        self.search_for_matches(new_pos+1, ds_pos+1, count);
                    },
                    _ => {
                        return;
                    }
                }
            },
            _ => {}
        }
    }
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let mut total: u32 = 0;

    each_line(input, |line| {
        let springs_condition = SpringsCondition::parse(line)?;
        total += springs_condition.find_matching_unknown_solutions();
        Ok(())
    })?;

    Ok(total.to_string())
}
