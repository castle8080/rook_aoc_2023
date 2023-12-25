use std::collections::HashSet;
use std::collections::HashMap;
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

    fn get_adjacent_nodes(&self, y: i32, x: i32) -> Vec<(i32, i32)> {
        let mut next_nodes: Vec<(i32, i32)> = Vec::new();

        match self.get(y, x) {
            Some(LocationType::SlopeUp) => {
                next_nodes.push((y - 1, x));
            },
            Some(LocationType::SlopeDown) => {
                next_nodes.push((y + 1, x));
            },
            Some(LocationType::SlopeLeft) => {
                next_nodes.push((y, x - 1));
            },
            Some(LocationType::SlopeRight) => {
                next_nodes.push((y, x + 1));
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
                            next_nodes.push((ny, nx));
                        }
                    }

                }
            },
            _ => {}
        }

        next_nodes
    }
}

pub struct LongestPathSolverBrute<'a> {
    pub trail: &'a HikingTrail,
    pub start: (i32, i32),
    pub end: (i32, i32),
}

impl<'a> LongestPathSolverBrute<'a> {

    pub fn new(trail: &'a HikingTrail, start: (i32, i32), end: (i32, i32)) -> Self {
        Self { trail, start, end }
    }

    pub fn search(&self) -> AOCResult<Vec<(i32, i32)>> {
        let mut path_stack: Vec<Vec<(i32, i32)>> = Vec::new();
        let mut visited: HashSet<(i32, i32)> = HashSet::new();

        visited.insert(self.start.clone());
        path_stack.push(vec![self.start.clone()]);

        let mut end_path: Option<Vec<(i32, i32)>> = None;

        while !path_stack.is_empty() {
            let last_path = &path_stack[path_stack.len() - 1];
            let last_node = &last_path[last_path.len() - 1];

            visited.insert(*last_node);

            let mut unwind = true;

            if *last_node == self.end {
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

    fn get_next_nodes(&self, (y, x): &(i32, i32), visited: &HashSet<(i32, i32)>) -> Vec<(i32, i32)> {
        let mut next_nodes: Vec<(i32, i32)> = Vec::new();

        let mut try_add = |y: i32, x: i32| {
            if !visited.contains(&(y, x)) {
                next_nodes.push((y, x));
            }
        };

        match self.trail.get(*y, *x) {
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

                    match self.trail.get(ny, nx) {
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
}

pub struct SimplifiedTrailSolver<'a> {
    // The trail to analyze.
    pub trail: &'a HikingTrail,

    // Where to start.
    pub start: (i32, i32),

    // Where to end.
    pub end: (i32, i32),

    // Simplified edges.
    pub edges: HashMap<(i32, i32), HashMap<(i32, i32), i32>>,

    // Keep track of longest path encountered.
    longest_path_cost: Option<i32>,
}

impl<'a> SimplifiedTrailSolver<'a> {
    pub fn new(trail: &'a HikingTrail, start: (i32, i32), end: (i32, i32)) -> Self {
        Self {
            trail, start, end,
            edges: HashMap::new(),
            longest_path_cost: None,
        }
    }

    pub fn solve(&mut self) -> AOCResult<i32> {
        self.simplify()?;
        self.verify()?;
        let mut visited: HashSet<(i32, i32)> = HashSet::new();
        visited.insert(self.start);

        self.search_longest(self.start, 0, &mut visited)?;
        self.longest_path_cost
            .ok_or_else(|| AOCError::ProcessingError("Could not find longest path.".into()))
    }

    fn on_end(&mut self, total_cost: i32) {
        match self.longest_path_cost {
            Some(c) => {
                if total_cost > c {
                    self.longest_path_cost = Some(total_cost);
                }
            },
            None => {
                self.longest_path_cost = Some(total_cost);
            }
        }
    }

    fn search_longest(&mut self, pos: (i32, i32), total_cost: i32, visited: &mut HashSet<(i32, i32)>) -> AOCResult<()> {

        if pos == self.end {
            self.on_end(total_cost);
        }

        visited.insert(pos);

        let mut explore_next: Vec<((i32, i32), i32)> = Vec::new();

        if let Some(dests) = self.edges.get(&pos) {
            for (next_pos, next_cost) in dests {
                if !visited.contains(next_pos) {
                    explore_next.push((next_pos.clone(), *next_cost));
                }
            }
        }

        for (next_pos, next_cost) in explore_next {
            self.search_longest(next_pos, total_cost + next_cost, visited)?;
        }

        visited.remove(&pos);

        Ok(())
    }

    fn verify(&self) -> AOCResult<()> {
        if !self.edges.contains_key(&self.end) {
            return Err(AOCError::ProcessingError("Could not find end on a trail path edge.".into()));
        }
        Ok(())
    }

    fn simplify(&mut self) -> AOCResult<()> {
        let mut visited: HashSet<(i32, i32)> = HashSet::new();
        visited.insert(self.start);

        let mut in_path: HashSet<(i32, i32)> = HashSet::new();
        in_path.insert(self.start);

        self.explore(self.start, self.start, 0, &mut visited, &mut in_path)?;

        Ok(())
    }

    fn on_found_edge(&mut self, start: (i32, i32), end: (i32, i32), cost: i32) {
        if cost > 0 {
            self.add_edge(start, end, cost);
            self.add_edge(end, start, cost);
        }
    }

    fn add_edge(&mut self, start: (i32, i32), end: (i32, i32), cost: i32) {
        match self.edges.get_mut(&start) {
            Some(ends) => {
                ends.insert(end, cost);
            },
            None => {
                let mut ends: HashMap<(i32, i32), i32> = HashMap::new();
                ends.insert(end, cost);
                self.edges.insert(start, ends);
            }
        }
    }

    fn explore(&mut self,
        start: (i32, i32),
        mut current: (i32, i32),
        mut cost: i32,
        visited: &mut HashSet<(i32, i32)>,
        in_path: &mut HashSet<(i32, i32)>)
        -> AOCResult<()>
    {
        loop {
            let adj_nodes: Vec<(i32, i32)> = self
                .trail
                .get_adjacent_nodes(current.0, current.1);

            // walking along path
            if adj_nodes.len() <= 2 {
                if let Some(new_node) = adj_nodes.iter().find(|n| !in_path.contains(n)) {
                    current = *new_node;
                    cost += 1;
                    visited.insert(current);
                    in_path.insert(current);
                }
                else {
                    self.on_found_edge(start, current, cost);
                    return Ok(())
                }
            }
            // At a junction
            else {
                self.on_found_edge(start, current, cost);
                for next_node in adj_nodes {
                    if !visited.contains(&next_node) {
                        visited.insert(next_node);
                        let mut new_in_path: HashSet<(i32, i32)> = HashSet::new();
                        new_in_path.insert(current);
                        new_in_path.insert(next_node);
                        self.explore(current, next_node, 1, visited, &mut new_in_path)?;
                    }
                }
                return Ok(())
            }
        }
    }

}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let trail = HikingTrail::parse(input)?;

    let start = trail.get_start()?;
    let end = trail.get_end()?;

    let lp_solver = LongestPathSolverBrute::new(&trail, start.clone(), end.clone());
    let end_path = lp_solver.search()?;

    // Subtract 1 to account for starting position
    let result = end_path.len() - 1;

    Ok(result.to_string())
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    let mut trail = HikingTrail::parse(input)?;

    trail.slopes_dont_matter();

    let start = trail.get_start()?;
    let end = trail.get_end()?;

    let mut o_solver = SimplifiedTrailSolver::new(&trail, start.clone(), end.clone());
    let result = o_solver.solve()?;

    Ok(result.to_string())
}