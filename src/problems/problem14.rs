use std::collections::HashMap;
use std::path::Path;

use crate::aocbase::{AOCResult, AOCError};
use crate::aocio::read_lines_as_bytes;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum RockType {
    Rounded = 0,
    Cube,
    Space,
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
    pub fn parse(c: char) -> AOCResult<RockType> {
        Ok(match c {
            'O' => RockType::Rounded,
            '#' => RockType::Cube,
            '.' => RockType::Space,
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
    pub rocks: Vec<Vec<RockType>>,
}

impl MirrorPlatform {

    pub fn new(width: usize, height: usize) -> MirrorPlatform {
        let mut rocks: Vec<Vec<RockType>> = Vec::new();
        for _ in 0..height {
            rocks.push(vec![RockType::Space; width]);
        }
        Self { width, height, rocks }
    }

    pub fn get(&self, y: usize, x: usize) -> Option<&RockType> {
        match self.rocks.get(y) {
            Some(row) => row.get(x),
            None => None,
        }
    }

    pub fn set(&mut self, y: usize, x: usize, rock_type: RockType) {
        self.rocks[y][x] = rock_type;
    }

    #[allow(dead_code)]
    pub fn render(&self) -> String {
        let mut output = String::new();

        for y in 0..self.height {
            for x in 0..self.width {
                let c = match self.get(y, x) {
                    Some(RockType::Cube) => '#',
                    Some(RockType::Rounded) => 'O',
                    _ => ' ',
                };
                output.push(c);
            }
            output.push('\n');
        }

        output
    }

    pub fn parse(input: impl AsRef<Path>) -> AOCResult<MirrorPlatform> {
        let map_data = read_lines_as_bytes(input)?;

        let width = map_data[0].len();
        let height = map_data.len();
        let mut mirror_platform = MirrorPlatform::new(width, height);

        for (y, row) in map_data.iter().enumerate() {
            for (x, c) in row.iter().enumerate() {
                mirror_platform.set(y, x, RockType::parse(*c as char)?);
            }
        }

        Ok(mirror_platform)
    }

    pub fn slide(&mut self, direction: Direction) {
        let slider = MirrorPlatformSlider::new(self, direction);
        slider.slide(self);
    }

    pub fn calculate_load(&self) -> usize {
        let mut load: usize = 0;
        for (y, row) in self.rocks.iter().enumerate() {
            for rock in row {
                if rock.is_rounded() {
                    load += self.height - y;
                }
            }
        }
        load
    }
}

pub struct MirrorPlatformSlider {
    start: i64,
    end: i64,
    delta: i64,

    outer_start: i64,
    outer_end: i64,

    getter: fn(this: &MirrorPlatform, i64, i64) -> &RockType,
    setter: fn(this: &mut MirrorPlatform, i64, i64, RockType),
}

impl MirrorPlatformSlider {

    pub fn new(mirror_platform: &MirrorPlatform, direction: Direction) -> Self {
        match direction {
            Direction::North => MirrorPlatformSlider {
                start: 0,
                end: mirror_platform.height as i64,
                delta: 1,
                outer_start: 0,
                outer_end: mirror_platform.width as i64,
                getter: Self::get_xy,
                setter: Self::set_xy,
            },
            Direction::East => MirrorPlatformSlider {
                start: mirror_platform.width as i64 - 1,
                end: -1,
                delta: -1,
                outer_start: 0,
                outer_end: mirror_platform.height as i64,
                getter: Self::get_yx,
                setter: Self::set_yx,
            },
            Direction::South => MirrorPlatformSlider {
                start: mirror_platform.height as i64 - 1,
                end: -1,
                delta: -1,
                outer_start: 0,
                outer_end: mirror_platform.width as i64,
                getter: Self::get_xy,
                setter: Self::set_xy,
            },
            Direction::West => MirrorPlatformSlider {
                start: 0,
                end: mirror_platform.width as i64,
                delta: 1,
                outer_start: 0,
                outer_end: mirror_platform.height as i64,
                getter: Self::get_yx,
                setter: Self::set_yx,
            }
        }
    }

    fn slide(&self, mirror_platform: &mut MirrorPlatform) {
        // Perform the actual slide
        for outer_pos in self.outer_start..self.outer_end {
            // The inner loop does the slide for a row or column.
            let mut inner_pos = self.start;
            let mut move_to: Option<i64> = None;
            
            while inner_pos != self.end {
                match (self.getter)(mirror_platform, outer_pos, inner_pos) {
                    RockType::Space => {
                        if let None = move_to {
                            move_to = Some(inner_pos);
                        }
                    },
                    RockType::Rounded => {
                        if let Some(move_pos) = move_to {
                            (self.setter)(mirror_platform, outer_pos, move_pos, RockType::Rounded);
                            (self.setter)(mirror_platform, outer_pos, inner_pos, RockType::Space);
                            move_to = Some(move_pos + self.delta);
                        }
                    },
                    RockType::Cube => {
                        move_to = None;
                    }
                }
                inner_pos += self.delta;
            }
        }
    }

    fn get_yx(mirror_platform: &MirrorPlatform, y: i64, x: i64) -> &RockType {
        mirror_platform.get(y as usize, x as usize).unwrap()
    }

    fn get_xy(mirror_platform: &MirrorPlatform, x: i64, y: i64) -> &RockType {
        Self::get_yx(mirror_platform, y, x)
    }

    fn set_yx(mirror_platform: &mut MirrorPlatform, y: i64, x: i64, rock_type: RockType) {
        mirror_platform.set(y as usize, x as usize, rock_type);
    }

    fn set_xy(mirror_platform: &mut MirrorPlatform, x: i64, y: i64, rock_type: RockType) {
        Self::set_yx(mirror_platform, y, x, rock_type)
    }
}

pub struct SpinTiltSolver {
    pub mirror_platform: MirrorPlatform,
    cycle_start: Option<i64>,
    cycle_end: Option<i64>,
    map_steps: HashMap<Vec<Vec<RockType>>, (i64, usize)>,
}

impl SpinTiltSolver {
    pub fn new(mirror_platform: MirrorPlatform) -> Self {
        SpinTiltSolver {
            mirror_platform,
            cycle_start: None,
            cycle_end: None,
            map_steps: HashMap::new(),
        }
    }

    pub fn find_cycle(&mut self) {
        let mut cycle = 1;

        self.cycle_start = None;
        self.cycle_end = None;
        self.map_steps = HashMap::new();

        while self.cycle_start.is_none() {
            self.run_cycle();
    
            match self.map_steps.get(&self.mirror_platform.rocks) {
                None => {
                    self.map_steps.insert(
                        self.mirror_platform.rocks.clone(),
                        (cycle, self.mirror_platform.calculate_load())
                    );
                },
                Some((prev, _)) => {
                    self.cycle_start = Some(*prev);
                    self.cycle_end = Some(cycle);
                }
            }
    
            cycle += 1;
        }
    }

    pub fn get_load(&self, cycle: i64) -> AOCResult<usize> {
        let cycle_target = match (self.cycle_start, self.cycle_end) {
            (Some(cycle_start), Some(cycle_end)) => {
                Ok((cycle - cycle_start) % (cycle_end - cycle_start) + cycle_start)
            },
            _ => Err(AOCError::ProcessingError("Have not found cycle start/end.".into())),
        }?;

        // Find the board for that target cycle and calculate the load.
        for (_, (cycle, load)) in &self.map_steps {
            if *cycle == cycle_target {
                return Ok(*load);
            }
        }

        Err(AOCError::ProcessingError("Could not find target cycle.".into()))
    }

    pub fn run_cycle(&mut self) {
        self.mirror_platform.slide(Direction::North);
        self.mirror_platform.slide(Direction::West);
        self.mirror_platform.slide(Direction::South);
        self.mirror_platform.slide(Direction::East);
    }
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let mut mirror_platform = MirrorPlatform::parse(input)?;
    mirror_platform.slide(Direction::North);

    let load = mirror_platform.calculate_load();
    Ok(load.to_string())
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    let mirror_platform = MirrorPlatform::parse(input)?;
    let mut solver = SpinTiltSolver::new(mirror_platform.clone());

    solver.find_cycle();

    let result = solver.get_load(1_000_000)?;

    Ok(result.to_string())
}