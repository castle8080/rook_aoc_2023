use std::collections::HashSet;
use std::path::Path;

use crate::aocbase::{AOCResult, AOCError};
use crate::aocio::read_lines_as_bytes;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LocationType {
    Path = 0,
    Forest,
    SlopeUp,
    SlopeRight,
    SlopeLeft,
    SlopeDown,
}

impl LocationType {
    pub fn from_char(c: char) -> AOCResult<LocationType> {
        use LocationType::*;
        Ok(match c {
            '.' => Path,
            '#' => Forest,
            '^' => SlopeUp,
            '>' => SlopeRight,
            'v' => SlopeDown,
            '<' => SlopeLeft,
            _ => {
                return Err(AOCError::ParseError(format!("Invalid character: {}", c)));
            }
        })
    }

    pub fn is_slope(&self) -> bool {
        use LocationType::*;
        match self {
            SlopeUp|SlopeDown|SlopeLeft|SlopeRight => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HikingTrail {
    map: Vec<Vec<LocationType>>,
}

impl HikingTrail {

    pub fn parse(input: impl AsRef<Path>) -> AOCResult<Self> { 
        let mut map: Vec<Vec<LocationType>> = Vec::new();

        for line in read_lines_as_bytes(input)? {
            map.push(line
                .iter()
                .map(|c| LocationType::from_char(*c as char))
                .collect::<AOCResult<Vec<LocationType>>>()?);
        }

        Ok(Self { map })
    }

    // Turn all slopes to paths.
    pub fn slopes_dont_matter(&mut self) {
        for row in &mut self.map {
            for cell in row {
                if cell.is_slope() {
                    *cell = LocationType::Path;
                }
            }
        }
    }

    pub fn get(&self, y: i32, x: i32) -> Option<LocationType> {
        if y >= 0 && (y as usize) < self.map.len() {
            let row = &self.map[y as usize];
            if x >= 0 && (x as usize) < row.len() {
                return Some(row[x as usize]);
            }
        }
        None
    }

    fn get_next_nodes(&self, (y, x): &(i32, i32), visited: &HashSet<(i32, i32)>) -> Vec<(i32, i32)> {
        let mut next_nodes: Vec<(i32, i32)> = Vec::new();

        let mut try_add = |y: i32, x: i32| {
            if !visited.contains(&(y, x)) {
                next_nodes.push((y, x));
            }
        };

        match self.get(*y, *x) {
            Some(LocationType::SlopeUp) => {
                try_add(y - 1, *x);
            },
            Some(LocationType::SlopeDown) => {
                try_add(y + 1, *x);
            },
            Some(LocationType::SlopeLeft) => {
                try_add(*y, x - 1);
            },
            Some(LocationType::SlopeRight) => {
                try_add(*y, x + 1);
            },

            Some(LocationType::Path) => {
                // Look for next nodes
                for (yd, xd) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let ny = y + yd;
                    let nx = x + xd;

                    match self.get(ny, nx) {
                        Some(LocationType::Forest) => {},
                        None => {},
                        _ => {
                            try_add(ny, nx);
                        }
                    }

                }
            },
            _ => {}
        }

        next_nodes
    }

    pub fn search_longest(&self, start: &(i32, i32), end: &(i32, i32)) -> AOCResult<Vec<(i32, i32)>> {
        let mut path_stack: Vec<Vec<(i32, i32)>> = Vec::new();
        let mut visited: HashSet<(i32, i32)> = HashSet::new();

        visited.insert(start.clone());
        path_stack.push(vec![start.clone()]);

        let mut end_path: Option<Vec<(i32, i32)>> = None;

        while !path_stack.is_empty() {
            let last_path = &path_stack[path_stack.len() - 1];
            let last_node = &last_path[last_path.len() - 1];

            visited.insert(*last_node);

            let mut unwind = true;

            if last_node == end {
                let last_max = match &end_path {
                    Some(ep) => ep.len() as i32,
                    None => 0,
                };
                if path_stack.len() as i32 > last_max {

                    let mut _end_path: Vec<(i32, i32)> = Vec::new();
                    for step in &path_stack {
                        _end_path.push(step[step.len() - 1]);
                    }
                    end_path = Some(_end_path);
                }
            }
            else {
                let next_nodes = self.get_next_nodes(&last_node, &visited);
                if next_nodes.len() > 0 {  
                    unwind = false;
                    path_stack.push(next_nodes);
                }
            }

            if unwind {
                while let Some(mut last_path) = path_stack.pop() {
                    if let Some(last_node) = last_path.pop() {
                        visited.remove(&last_node);
                    }
                    if !last_path.is_empty() {
                        path_stack.push(last_path);
                        break;
                    }
                }
            }
        }

        end_path.ok_or_else(|| AOCError::ProcessingError("Could not find path.".into()))
    }

    pub fn get_start(&self) -> AOCResult<(i32, i32)> {
        self.map[0]
            .iter()
            .enumerate()
            .find(|(_, lt)| **lt == LocationType::Path)
            .map(|(i, _)| (0, i as i32))
            .ok_or_else(|| AOCError::ProcessingError(format!("Couldn't find start.")))
    }

    pub fn get_end(&self) -> AOCResult<(i32, i32)> {
        self.map[self.map.len() - 1]
            .iter()
            .enumerate()
            .find(|(_, lt)| **lt == LocationType::Path)
            .map(|(i, _)| ((self.map.len() - 1) as i32, i as i32))
            .ok_or_else(|| AOCError::ProcessingError(format!("Couldn't find end.")))
    }
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let trail = HikingTrail::parse(input)?;

    let start = trail.get_start()?;
    let end = trail.get_end()?;

    let end_path = trail.search_longest(&start, &end)?;

    // Subtract 1 to account for starting position
    let result = end_path.len() - 1;

    Ok(result.to_string())
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    let mut trail = HikingTrail::parse(input)?;

    trail.slopes_dont_matter();

    let start = trail.get_start()?;
    let end = trail.get_end()?;

    let end_path = trail.search_longest(&start, &end)?;

    // Subtract 1 to account for starting position
    let result = end_path.len() - 1;

    Ok(result.to_string())
}