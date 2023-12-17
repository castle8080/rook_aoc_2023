use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::path::Path;

use crate::aocio::read_lines_as_bytes;
use crate::aocbase::{AOCResult, AOCError};

fn num_from_char(c: char) -> AOCResult<i32> {
    if c >= '0' && c <= '9' {
        Ok(c as i32 - '0' as i32)
    }
    else {
        Err(AOCError::ParseError(format!("Invalid number character: {}", c)))
    }
}

#[derive(Debug, Clone)]
pub struct HeatLossMap {
    map: Vec<Vec<i32>>,
}

impl HeatLossMap {

    pub fn width(&self) -> i32 {
        self.map[0].len() as i32
    }

    pub fn height(&self) -> i32 {
        self.map.len() as i32
    }

    pub fn get_value(&self, y: i32, x: i32) -> i32 {
        self.map[y as usize][x as usize]
    }

    pub fn parse(input: impl AsRef<Path>) -> AOCResult<Self> {
        let lines = read_lines_as_bytes(input)?;
        let mut map: Vec<Vec<i32>> = Vec::new();

        for line in lines {
            map.push(line
                .iter()
                .map(|c| num_from_char(*c as char))
                .collect::<AOCResult<Vec<i32>>>()?);
        }

        Ok(HeatLossMap { map })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(&self, other: &Direction) -> bool {
        use Direction::*;
        match (self, other) {
            (Up, Down)|(Down, Up)|(Left, Right)|(Right, Left) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathFindState {
    pub heat_loss: i32,
    pub direction: Direction,
    pub direction_count: i32,
    pub y: i32,
    pub x: i32,
}

impl PathFindState {

    pub fn apply(&self, direction: &Direction) -> PathFindState {
        let mut new_st = self.clone();

        if *direction == self.direction {
            new_st.direction_count = self.direction_count + 1
        }
        else {
            new_st.direction_count = 1;
        };

        match direction {
            Direction::Up => {
                new_st.y -= 1;
            },
            Direction::Down => {
                new_st.y += 1;
            },
            Direction::Right => {
                new_st.x += 1;
            },
            Direction::Left => {
                new_st.x -= 1;
            }
        }

        new_st.direction = direction.clone();
        new_st
    }

}

impl PartialOrd for PathFindState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathFindState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.heat_loss * -1, &self.direction, self.direction_count)
            .cmp(&(other.heat_loss * -1, &other.direction, other.direction_count))
    }
}

impl PathFindState {
    pub fn new(heat_loss: i32, direction: Direction, direction_count: i32, y: i32, x: i32) -> Self {
        Self { heat_loss, direction, direction_count, y, x }
    }
}

pub struct HLPathFinder<'a> {
    heat_loss_map: &'a HeatLossMap,
    end: (i32, i32),
    path_find_states: BinaryHeap<PathFindState>,
    known_states: HashMap<(i32, i32, Direction, i32), i32>,
} 

impl<'a> HLPathFinder<'a> {
    pub fn new(heat_loss_map: &'a HeatLossMap, end: (i32, i32)) -> Self {
        Self {
            heat_loss_map,
            end,
            path_find_states: BinaryHeap::new(),
            known_states: HashMap::new(),
        }
    }

    fn add_state(&mut self, pf_st: PathFindState) {
        let key = (pf_st.y, pf_st.x, pf_st.direction.clone(), pf_st.direction_count);
        if let Some(hl) = self.known_states.get(&key) {
            if *hl <= pf_st.heat_loss {
                return;
            }
        }
        self.known_states.insert(key, pf_st.heat_loss);
        self.path_find_states.push(pf_st);
    }

    pub fn find(&mut self, (y, x): (i32, i32)) -> AOCResult<PathFindState> {
        self.add_state(PathFindState::new(0, Direction::Down, 0, y, x));

        let directions = vec![
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];

        let width = self.heat_loss_map.width();
        let height = self.heat_loss_map.height();

        while let Some(pf_st) = self.path_find_states.pop() {
            //println!("Looking at: {:?}", pf_st);

            // Found end state
            if pf_st.y == self.end.0 && pf_st.x == self.end.1 {
                return Ok(pf_st);
            }

            for d in &directions {
                // Skip going back
                if d.opposite(&pf_st.direction) {
                    continue;
                }

                let mut next_pf_st = pf_st.apply(d);

                if next_pf_st.y >= 0 &&
                    next_pf_st.y < height &&
                    next_pf_st.x >= 0 &&
                    next_pf_st.x < width &&
                    next_pf_st.direction_count <= 3
                {
                    let hl = self.heat_loss_map.get_value(next_pf_st.y, next_pf_st.x);

                    // Add up heat loss
                    next_pf_st.heat_loss += hl;

                    // Push onto heap search states.
                    self.add_state(next_pf_st);
                }
            }

        }

        Err(AOCError::ProcessingError("Could not find path.".into()))
    }
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let hl_map = HeatLossMap::parse(input)?;
    let mut path_finder = HLPathFinder::new(&hl_map, (hl_map.height() - 1, hl_map.width() - 1));
    let result = path_finder.find((0, 0))?;

    Ok(result.heat_loss.to_string())
}
