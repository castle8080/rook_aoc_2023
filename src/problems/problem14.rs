use std::collections::HashMap;
use std::path::Path;
use std::mem::replace;

use crate::aocbase::{AOCResult, AOCError};
use crate::aocio::read_lines_as_bytes;

#[derive(Debug, PartialEq, Eq, Hash)]
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

#[derive(Debug)]
pub struct Rock {
    pub kind: RockType,
    pub x: usize,
    pub y: usize,
}

#[derive(Debug)]
pub struct MirrorPlatform {
    pub width: usize,
    pub height: usize,
    pub columns: HashMap<usize, Vec<Rock>>,
}

impl MirrorPlatform {

    pub fn render(&self) -> String {
        let mut output = String::new();
        let rock_index = self.get_rock_index();

        for y in 0..self.height {
            for x in 0..self.width {
                match rock_index.get(&(y, x)) {
                    None => output.push(' '),
                    Some(rock) => {
                        if rock.kind.is_rounded() {
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

    pub fn get_rock_index<'a>(&'a self) -> HashMap<(usize, usize), &'a Rock> {
        let mut index: HashMap<(usize, usize), &'a Rock> = HashMap::new();

        for column in self.columns.values() {
            for rock in column {
                index.insert((rock.y, rock.x), rock);
            }
        }

        index
    }

    pub fn parse(input: impl AsRef<Path>) -> AOCResult<MirrorPlatform> {
        let map_data = read_lines_as_bytes(input)?;

        let width = map_data[0].len();
        let height = map_data.len();
        let mut columns: HashMap<usize, Vec<Rock>> = HashMap::new();

        for (y, row) in map_data.iter().enumerate() {
            for (x, c) in row.iter().enumerate() {
                if let Some(kind) = RockType::parse(*c as char)? {
                    let column = match columns.get_mut(&x) {
                        Some(column) => column,
                        None => {
                            let column: Vec<Rock> = Vec::new();
                            columns.insert(x, column);
                            columns.get_mut(&x).unwrap()
                        }
                    };

                    column.push(Rock { y, x, kind });
                }
            }
        }

        Ok(MirrorPlatform { height, width, columns })
    }

    pub fn slide(&mut self) {
        for (_, column) in &mut self.columns {
            Self::slide_column(column);
        }
    }

    fn slide_column(column: &mut Vec<Rock>) {
        let mut last_pos = 0;
        for rock in column {
            if rock.y > last_pos && rock.kind.is_rounded() {
                rock.y = last_pos;
                last_pos += 1;
            }
            else {
                last_pos = rock.y + 1;
            }
        }
    }

    pub fn calculate_load(&self) -> usize {
        let mut load: usize = 0;

        for column in self.columns.values() {
            for rock in column {
                if rock.kind.is_rounded() {
                    load += self.height - rock.y;
                }
            }
        }

        load
    }

}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let mut mirror_platform = MirrorPlatform::parse(input)?;

    println!("{}", mirror_platform.render());

    mirror_platform.slide();
    let load = mirror_platform.calculate_load();

    println!("{}", mirror_platform.render());


    Ok(load.to_string())
}