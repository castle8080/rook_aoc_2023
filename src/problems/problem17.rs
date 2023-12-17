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
    Up = 0,
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

pub trait HLPathFinderRules {

    fn is_endable(&self,
        path_finder: &HLPathFinder,
        pf_st: &PathFindState) -> bool;

    fn check_direction(&self,
        path_finder: &HLPathFinder,
        pf_st: &PathFindState,
        d: &Direction) -> bool;

    fn check_prune(&self,
        path_finder: &HLPathFinder,
        pf_st: &PathFindState,
        direction_tracking: &HashMap<i32, i32>) -> bool;
}

pub struct HLPathFinder<'a>
{
    heat_loss_map: &'a HeatLossMap,
    end: (i32, i32),
    path_find_states: BinaryHeap<PathFindState>,
    known_states: HashMap<(i32, i32, Direction), HashMap<i32, i32>>,
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

    fn add_state(&mut self, pf_st: PathFindState, rules: &impl HLPathFinderRules) {
        let key = (pf_st.y, pf_st.x, pf_st.direction.clone());

        match self.known_states.get(&key) {
            None => {
                self.known_states.insert(key.clone(), HashMap::new());
            },
            Some(direction_tracking) => {
                if rules.check_prune(self, &pf_st, direction_tracking) {
                    return;
                }
                /* 
                for (k_d_count, k_hl) in pos_dir_map {
                    if *k_d_count >= 4 {
                        if *k_d_count <= pf_st.direction_count && *k_hl <= pf_st.heat_loss {
                            return;
                        }
                    }
                    else {

                        if *k_d_count == pf_st.direction_count && *k_hl <= pf_st.heat_loss {
                            return;
                        }
                    }
                }
                */
            }
        };

        // The code above should guarantee key exists.
        self.known_states
            .get_mut(&key)
            .unwrap()
            .insert(pf_st.direction_count, pf_st.heat_loss);

        self.path_find_states.push(pf_st);
    }

    pub fn find(&mut self, (y, x): (i32, i32), rules: &impl HLPathFinderRules) -> AOCResult<PathFindState> {
        self.add_state(PathFindState::new(0, Direction::Down, 0, y, x), rules);

        let directions = vec![
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];

        let width = self.heat_loss_map.width();
        let height = self.heat_loss_map.height();

        while let Some(pf_st) = self.path_find_states.pop() {
            // Found end state
            if pf_st.y == self.end.0 && pf_st.x == self.end.1 &&
                rules.is_endable(self, &pf_st)
            {
                return Ok(pf_st);
            }

            for d in &directions {
                if !rules.check_direction(self, &pf_st, &d) {
                    continue;
                }

                let mut next_pf_st = pf_st.apply(d);

                if next_pf_st.y >= 0 &&
                    next_pf_st.y < height &&
                    next_pf_st.x >= 0 &&
                    next_pf_st.x < width
                {
                    let hl = self.heat_loss_map.get_value(next_pf_st.y, next_pf_st.x);

                    // Add up heat loss
                    next_pf_st.heat_loss += hl;

                    // Push onto heap search states.
                    self.add_state(next_pf_st, rules);
                }
            }

        }

        Err(AOCError::ProcessingError("Could not find path.".into()))
    }
}

pub struct Part1PathFinderRules {
}

impl Part1PathFinderRules {
    pub fn new() -> Self {
        Self {}
    }
}

impl HLPathFinderRules for Part1PathFinderRules {
    
    fn is_endable(&self,
        _path_finder: &HLPathFinder,
        _pf_st: &PathFindState) -> bool
    {
        true
    }

    fn check_direction(&self,
        _path_finder: &HLPathFinder,
        pf_st: &PathFindState,
        d: &Direction) -> bool
    {
        !pf_st.direction.opposite(d) &&
            (pf_st.direction_count < 3 || pf_st.direction != *d)
    }

    fn check_prune(&self,
        _path_finder: &HLPathFinder,
        pf_st: &PathFindState,
        direction_tracking: &HashMap<i32, i32>) -> bool
    {
        for (k_d_count, k_hl) in direction_tracking {
            if *k_d_count <= pf_st.direction_count && *k_hl <= pf_st.heat_loss {
                return true;
            }
        }
        false
    }
}

pub struct Part2PathFinderRules {
}

impl Part2PathFinderRules {
    pub fn new() -> Self {
        Self {}
    }
}

impl HLPathFinderRules for Part2PathFinderRules {
    
    fn is_endable(&self, _path_finder: &HLPathFinder, pf_st: &PathFindState) -> bool {
        pf_st.direction_count >= 4
    }

    fn check_direction(&self, _path_finder: &HLPathFinder, pf_st: &PathFindState, d: &Direction) -> bool {
        if pf_st.direction.opposite(d) {
            false
        }
        else if pf_st.direction_count < 4 {
            *d == pf_st.direction
        }
        else if pf_st.direction_count >= 10 {
            *d != pf_st.direction
        }
        else {
            true
        }
    }

    fn check_prune(&self,
        _path_finder: &HLPathFinder,
        pf_st: &PathFindState,
        direction_tracking: &HashMap<i32, i32>) -> bool
    {
        for (k_d_count, k_hl) in direction_tracking {
            // If it has gone at least 4 it had some choice.
            if *k_d_count >= 4 {
                if *k_d_count <= pf_st.direction_count && *k_hl <= pf_st.heat_loss {
                    return true;
                }
            }
            // Otherwise just use current count
            else if *k_d_count == pf_st.direction_count && *k_hl <= pf_st.heat_loss {
                return true;
            }
        }
        false
    }
}

pub fn run_part(input: impl AsRef<Path>, rules: impl HLPathFinderRules) -> AOCResult<String> {
    let hl_map = HeatLossMap::parse(input)?;
    let mut path_finder = HLPathFinder::new(&hl_map, (hl_map.height() - 1, hl_map.width() - 1));
    let result = path_finder.find((0, 0), &rules)?;

    Ok(result.heat_loss.to_string())
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    run_part(input, Part1PathFinderRules::new())
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    run_part(input, Part2PathFinderRules::new())
}
