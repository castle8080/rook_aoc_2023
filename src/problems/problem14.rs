use std::collections::HashMap;
use std::path::Path;

use crate::aocbase::{AOCResult, AOCError};
use crate::aocio::read_lines_as_bytes;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum RockType {
    Rounded = 0,
    Cube,
}

impl RockType {
    pub fn is_rounded(&self) -> bool {
        match self {
            RockType::Rounded => true,
            _ => false,
        }
    }
}

impl RockType {
    pub fn parse(c: char) -> AOCResult<Option<RockType>> {
        Ok(match c {
            'O' => Some(RockType::Rounded),
            '#' => Some(RockType::Cube),
            '.' => None,
            _ => {
                return Err(AOCError::ParseError(format!("Invalid rock type: ({})", c)));
            }
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Direction {
    North = 0,
    East,
    South,
    West,
}

#[derive(Debug, Clone)]
pub struct MirrorPlatform {
    pub width: usize,
    pub height: usize,
    pub rocks: HashMap<(usize, usize), RockType>,
}

impl MirrorPlatform {

    #[allow(dead_code)]
    pub fn render(&self) -> String {
        let mut output = String::new();

        for y in 0..self.height {
            for x in 0..self.width {
                match self.rocks.get(&(y, x)) {
                    None => output.push(' '),
                    Some(rock) => {
                        if rock.is_rounded() {
                            output.push('O');
                        }
                        else {
                            output.push('#');
                        }
                    }
                }
            }
            output.push('\n');
        }

        output
    }

    pub fn parse(input: impl AsRef<Path>) -> AOCResult<MirrorPlatform> {
        let map_data = read_lines_as_bytes(input)?;

        let width = map_data[0].len();
        let height = map_data.len();
        let mut rocks: HashMap<(usize, usize), RockType> = HashMap::new();

        for (y, row) in map_data.iter().enumerate() {
            for (x, c) in row.iter().enumerate() {
                if let Some(kind) = RockType::parse(*c as char)? {
                    rocks.insert((y, x), kind);
                }
            }
        }

        Ok(MirrorPlatform { height, width, rocks })
    }

    pub fn slide(&mut self, direction: Direction) {
        // Setup common variables for iterating, getting, and setting
        // values while sliding. This is to keep from repeating similar
        // code for each direction.
        let start: i64;
        let end: i64;
        let delta: i64;

        let outer_start: i64;
        let outer_end: i64;

        let getter: fn(this: &MirrorPlatform, i64, i64) -> Option<&RockType>;
        let setter: fn(this: &mut MirrorPlatform, i64, i64, Option<RockType>);

        match direction {
            Direction::North => {
                start = 0;
                end = self.height as i64;
                delta = 1;
                outer_start = 0;
                outer_end = self.width as i64;
                getter = Self::get_xy;
                setter = Self::set_xy;
            },
            Direction::East => {
                start = self.width as i64 - 1;
                end = -1;
                delta = -1;
                outer_start = 0;
                outer_end = self.height as i64;
                getter = Self::get_yx;
                setter = Self::set_yx;
            },
            Direction::South => {
                start = self.height as i64 - 1;
                end = -1;
                delta = -1;
                outer_start = 0;
                outer_end = self.width as i64;
                getter = Self::get_xy;
                setter = Self::set_xy;
            },
            Direction::West => {
                start = 0;
                end = self.width as i64;
                delta = 1;
                outer_start = 0;
                outer_end = self.height as i64;
                getter = Self::get_yx;
                setter = Self::set_yx;
            }
        }

        // Perform the actual slide
        for outer_pos in outer_start..outer_end {
            // The inner loop does the slide for a row or column.
            let mut inner_pos = start;
            let mut move_to: Option<i64> = None;
            
            while inner_pos != end {
                match getter(self, outer_pos, inner_pos) {
                    None => {
                        if let None = move_to {
                            move_to = Some(inner_pos);
                        }
                    },
                    Some(RockType::Rounded) => {
                        if let Some(move_pos) = move_to {
                            setter(self, outer_pos, move_pos, Some(RockType::Rounded));
                            setter(self, outer_pos, inner_pos, None);
                            move_to = Some(move_pos + delta);
                        }
                    },
                    Some(RockType::Cube) => {
                        move_to = None;
                    }
                }
                inner_pos += delta;
            }
        }
    }

    fn get_yx(&self, y: i64, x: i64) -> Option<&RockType> {
        self.rocks.get(&(y as usize, x as usize))
    }

    fn get_xy(&self, x: i64, y: i64) -> Option<&RockType> {
        self.get_yx(y, x)
    }

    fn set_yx(&mut self, y: i64, x: i64, rock_type: Option<RockType>) {
        match rock_type {
            None => self.rocks.remove(&(y as usize, x as usize)),
            Some(rock_type) => self.rocks.insert((y as usize, x as usize), rock_type),
        };
    }

    fn set_xy(&mut self, x: i64, y: i64, rock_type: Option<RockType>) {
        self.set_yx(y, x, rock_type)
    }

    pub fn calculate_load(&self) -> usize {
        let mut load: usize = 0;
        for ((y, _), rock_type) in &self.rocks {
            if rock_type.is_rounded() {
                load += self.height - y;
            }
        }
        load
    }
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let mut mirror_platform = MirrorPlatform::parse(input)?;
    mirror_platform.slide(Direction::North);

    let load = mirror_platform.calculate_load();
    Ok(load.to_string())
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    let mut mirror_platform = MirrorPlatform::parse(input)?;
    let directions = vec![Direction::North, Direction::West, Direction::South, Direction::East];

    // Using the rendered board as a key.
    // I tried using a HashMap as a key itself and ran into issues.
    // I see why this is an issue as you would need to sort to make a consistent key.
    // I could try moving to a BTree or using Vec<Vec<RockType>> instead of trying to
    // make it all sparse.
    let mut board_cycles: HashMap<String, (i64, MirrorPlatform)> = HashMap::new();

    let mut cycle_start: Option<i64> = None;
    let mut cycle_end: Option<i64> = None;

    // Find a sequence where running a cycle gets back to a known state.
    // Store the boards (mirror_platforms) for the cycles through the
    // cycling sequence of configurations.
    let mut cycle = 1;
    while cycle_start.is_none() {
        for d in &directions {
            mirror_platform.slide(*d);
        }

        let s = mirror_platform.render();
        match board_cycles.get(&s) {
            None => {
                board_cycles.insert(s, (cycle, mirror_platform.clone()));
            },
            Some((prev, _)) => {
                cycle_start = Some(*prev);
                cycle_end = Some(cycle);
            }
        }

        cycle += 1;
    }

    let cycle_start = cycle_start
        .ok_or_else(|| AOCError::ProcessingError("Could not find cycle start.".into()))?;

    let cycle_end = cycle_end
        .ok_or_else(|| AOCError::ProcessingError("Could not find cycle end.".into()))?;

    // Use the start and end to figure out the equivalent state
    // for the 1000000th value.

    let cycle_target = (1_000_000_000 - cycle_start) % (cycle_end - cycle_start) + cycle_start;

    // Find the board for that target cycle and calculate the load.
    for (_, (cycle, mp)) in board_cycles {
        if cycle == cycle_target {
            let result = mp.calculate_load();
            return Ok(result.to_string());
        }
    }

    Err(AOCError::ProcessingError("Could not find target cycle.".into()))
}