use std::collections::HashSet;
use std::path::Path;

use crate::aocbase::{AOCResult, AOCError};
use crate::aocio::read_lines_as_bytes;

//paths (.), forest (#), and steep slopes (^, >, v, and <).

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
}

#[derive(Debug, Clone)]
pub struct SearchNode {
    visited: HashSet<(i32, i32)>,
    last: (i32, i32),
}

impl SearchNode {

    pub fn single(pos: (i32, i32)) -> Self {
        let mut _self = Self { visited: HashSet::new(), last: pos };
        _self.visited.insert(_self.last);
        _self
    }

    pub fn add(&mut self, pos: (i32, i32)) {
        if self.visited.contains(&pos) {
            panic!("bad!");
        }
        self.last = pos.clone();
        self.visited.insert(pos);
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

    pub fn get(&self, y: i32, x: i32) -> Option<LocationType> {
        if y >= 0 && (y as usize) < self.map.len() {
            let row = &self.map[y as usize];
            if x >= 0 && (x as usize) < row.len() {
                return Some(row[x as usize]);
            }
        }
        None
    }

    fn get_next_nodes(&self, node: &SearchNode) -> Vec<(i32, i32)> {
        let mut next_nodes: Vec<(i32, i32)> = Vec::new();

        let mut try_add = |y: i32, x: i32| {
            if !node.visited.contains(&(y, x)) {
                next_nodes.push((y, x));
            }
        };

        match self.get(node.last.0, node.last.1) {
            Some(LocationType::SlopeUp) => {
                try_add(node.last.0 - 1, node.last.1);
            },
            Some(LocationType::SlopeDown) => {
                try_add(node.last.0 + 1, node.last.1);
            },
            Some(LocationType::SlopeLeft) => {
                try_add(node.last.0, node.last.1 - 1);
            },
            Some(LocationType::SlopeRight) => {
                try_add(node.last.0, node.last.1 + 1);
            },

            Some(LocationType::Path) => {
                // Look for next nodes
                for (yd, xd) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let ny = node.last.0 + yd;
                    let nx = node.last.1 + xd;

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

    pub fn find_end_paths(&self, start: &(i32, i32), end: &(i32, i32)) -> Vec<SearchNode> {
        let mut node_stack = vec![SearchNode::single(start.clone())];
        let mut end_search_nodes: Vec<SearchNode> = Vec::new();

        while let Some(mut node) = node_stack.pop() {
            //println!("on: {:?}", node);

            // Found end
            if node.last == *end {
                end_search_nodes.push(node);
                continue;
            }

            let next_nodes = self.get_next_nodes(&node);

            // For 1 choice no need to clone
            if next_nodes.len() == 1 {
                node.add(next_nodes[0]);
                node_stack.push(node);
            }
            else {
                for next_node in next_nodes {
                    let mut new_node = node.clone();
                    new_node.add(next_node);
                    node_stack.push(new_node);
                }
            }
        }

        end_search_nodes
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

    let end_nodes = trail.find_end_paths(&start, &end);
    let end_node = end_nodes
        .iter()
        .max_by_key(|node| node.visited.len())
        .ok_or_else(|| AOCError::ProcessingError("No path found.".into()))?;


    // Subtract 1 to account for starting position
    let result = end_node.visited.len() - 1;

    Ok(result.to_string())
}