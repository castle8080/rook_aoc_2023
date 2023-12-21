use std::collections::HashSet;
use std::collections::VecDeque;
use std::path::Path;

use crate::aocbase::{AOCResult, AOCError};
use crate::aocio::read_lines_as_bytes;

#[derive(Debug, Copy, Clone)]
pub enum Space {
    Start = 0,
    Plot,
    Rock,
}

impl Space {
    pub fn from_char(c: char) -> AOCResult<Space> {
        use Space::*;
        Ok(match c {
            'S' => Start,
            '.' => Plot,
            '#' => Rock,
            _ => return Err(AOCError::ParseError(format!("Invalid character: {}", c)))
        })
    }
}

#[derive(Debug, Clone)]
pub struct Garden {
    pub map: Vec<Vec<Space>>,
}

impl Garden {

    pub fn new(map: Vec<Vec<Space>>) -> Garden {
        Self { map }
    }

    pub fn width(&self) -> i32 {
        self.map[0].len() as i32
    }

    pub fn height(&self) -> i32 {
        self.map.len() as i32
    }

    pub fn get(&self, y: i32, x: i32) -> Option<Space> {
        if y >= 0 && y < self.height() && x >= 0 && x < self.width() {
            Some(self.map[y as usize][x as usize])
        }
        else {
            None
        }
    }

    pub fn find_possible_end_positions(&self, start_y: i32, start_x: i32, steps: i32) -> Vec<(i32, i32)> {
        let explored = self.explore(start_y, start_x, steps);

        explored
            .iter()
            .filter(|(_, _, pos_steps)| *pos_steps == steps)
            .map(|(y, x, _)| (*y, *x))
            .collect()
    }

    pub fn explore(&self, start_y: i32, start_x: i32, steps: i32) -> HashSet<(i32, i32, i32)> {
        let mut x_queue: VecDeque<(i32, i32, i32)> = VecDeque::new();
        let mut visited: HashSet<(i32, i32, i32)> = HashSet::new();

        match self.get(start_y, start_x) {
            None|Some(Space::Rock) => return visited,
            _ => {}
        }

        x_queue.push_back((start_y, start_x, 0));
        visited.insert((start_y, start_x, 0));

        while let Some((cur_y, cur_x, cur_steps)) = x_queue.pop_front() {
            for (yd, xd) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let adj_y = cur_y + yd;
                let adj_x = cur_x + xd;
                let adj_steps = cur_steps + 1;
                match self.get(adj_y, adj_x) {
                    Some(Space::Start|Space::Plot) => {
                        if adj_steps <= steps && visited.insert((adj_y, adj_x, adj_steps)) {
                            x_queue.push_back((adj_y, adj_x, adj_steps));
                        }
                    },
                    _ => {}
                }
            }
        }

        visited
    }

    pub fn find_start(&self) -> AOCResult<(i32, i32)> {
        for (y, row) in self.map.iter().enumerate() {
            for (x, space) in row.iter().enumerate() {
                if let Space::Start = space {
                    return Ok((y as i32, x as i32));
                }
            }
        }
        Err(AOCError::ProcessingError("No start position found.".into()))
    }

    pub fn parse(input: impl AsRef<Path>) -> AOCResult<Garden> {
        let mut map: Vec<Vec<Space>> = Vec::new();
        let lines = read_lines_as_bytes(input)?;

        for line in lines {
            map.push(line
                .iter()
                .map(|c| Space::from_char(*c as char))
                .collect::<AOCResult<Vec<Space>>>()?);
        }

        Ok(Garden::new(map))
    }
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let garden = Garden::parse(input)?;

    let (start_y, start_x) = garden.find_start()?;
    let visited = garden.find_possible_end_positions(start_y, start_x, 64);
    let result = visited.len();

    Ok(result.to_string())
}