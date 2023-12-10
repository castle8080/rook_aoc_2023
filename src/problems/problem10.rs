use std::path::Path;
use std::collections::HashSet;

use crate::aocbase::{AOCResult, AOCError};
use crate::aocio::read_lines_as_bytes;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub enum Pipe {
    Start = 0,
    NorthSouth,
    EastWest,
    NorthEast,
    NorthWest,
    SouthWest,
    SouthEast,
    Ground,
}

macro_rules! make_has_dir_method {
    ($method:ident => $d1:ident|$d2:ident|$d3:ident) => {
        pub fn $method(&self) -> bool {
            use Pipe::*;
            match self {
                Start|$d1|$d2|$d3 => true,
                _ => false,
            }
        }
    }
}

impl Pipe {

    pub fn from_char(c: char) -> AOCResult<Pipe> {
        use Pipe::*;
        Ok(match c {
            '|' => NorthSouth,
            '-' => EastWest,
            'L' => NorthEast,
            'J' => NorthWest,
            '7' => SouthWest,
            'F' => SouthEast,
            '.' => Ground,
            'S' => Start,
            _ => { return Err(AOCError::ParseError(format!("Invalid character for Pipe: {}", c))); }
        })
    }

    make_has_dir_method!(has_north => NorthSouth|NorthEast|NorthWest);
    make_has_dir_method!(has_south => NorthSouth|SouthEast|SouthWest);
    make_has_dir_method!(has_east => EastWest|NorthEast|SouthEast);
    make_has_dir_method!(has_west => EastWest|NorthWest|SouthWest);
}

#[derive(Debug)]
pub struct PipeMap {
    pub map: Vec<Vec<Pipe>>,
}

impl PipeMap {

    pub fn new(map: Vec<Vec<Pipe>>) -> AOCResult<Self> {
        // Validate the data
        if map.len() == 0 {
            return Err(AOCError::ParseError("Empty map.".into()));
        }
        let width = map[0].len();
        for m in map.iter() {
            if m.len() != width {
                return Err(AOCError::ParseError(format!("Jagged map! width={}, have={}", width, m.len())));
            }
        }

        Ok(PipeMap { map })
    }

    pub fn width(&self) -> usize {
        self.map[0].len()
    }

    pub fn height(&self) -> usize {
        self.map.len()
    }

    fn get_connected_positions(&self, (h, w): (usize, usize)) -> Vec<(usize, usize)> {
        let mut connections: Vec<(usize, usize)> = Vec::new();

        let max_h = self.height() as i64;
        let max_w = self.width() as i64;

        for (h_delta, w_delta) in vec![(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let (nh, nw) = (h as i64 + h_delta, w as i64 + w_delta);
            if nh >= 0 && nh < max_h && nw >= 0 && nw < max_w {
                let (nh, nw) = (nh as usize, nw as usize);
                if self.is_connected((h, w), (nh, nw)) {
                    connections.push((nh as usize, nw as usize));
                }
            }
        }

        connections
    }

    pub fn get_enclosure_path(&self, (start_h, start_w): (usize, usize)) -> Option<Vec<(usize, usize)>> {
        let start_pos = (start_h, start_w);
        let paths = self.search_paths(start_pos, start_pos);

        for path in paths {
            // You need more than 3 nodes in the path for a loop.
            // this would be starting at one, going 1, and going back.
            if path.len() > 3 {
                // Just return the first path enclosuer.
                // I suppose there could be more than 1?
                return Some(path);
            }
        }

        None
    }

    pub fn search_paths(&self, (start_h, start_w): (usize, usize), (end_h, end_w): (usize, usize))
        -> Vec<Vec<(usize, usize)>> 
    {
        // This would probably be easier to read with recursion.
        let mut search_path = SearchPath::new();
        search_path.add((start_h, start_w));

        let mut search_stack: Vec<SearchPath> = vec![search_path];
        let mut wanted_paths: Vec<Vec<(usize, usize)>> = Vec::new();
        let mut branches: Vec<(usize, usize)> = Vec::new();

        while let Some(mut search_path) = search_stack.pop() {
            let (cur_h, cur_w) = search_path.path.last().unwrap();

            branches.clear();

            for (next_h, next_w) in self.get_connected_positions((*cur_h, *cur_w)) {
                // Is this a target!
                if next_h == end_h && next_w == end_w {
                    let mut path = search_path.path.clone();
                    path.push((next_h, next_w));
                    wanted_paths.push(path);
                }

                // One we want to visit
                else if !search_path.has_visited(&(next_h as usize, next_w as usize)) {
                    branches.push((next_h, next_w));
                }
            }

            // For more than 1 branch we need to clone.
            for branch in branches.iter().skip(1) {
                let mut branch_search_path = search_path.clone();
                branch_search_path.add(*branch);
                search_stack.push(branch_search_path);
            }

            // Don't clone if we don't have to
            if let Some(branch) = branches.get(0) {
                search_path.add(*branch);
                search_stack.push(search_path);
            }
        }

        wanted_paths
    }

    pub fn is_connected(&self, (h1, w1): (usize, usize), (h2, w2): (usize, usize)) -> bool {
        let p1 = self.map[h1][w1];
        let p2 = self.map[h2][w2];

        match ((h2 as i64 - h1 as i64), (w2 as i64 - w1 as i64)) {
            (-1, 0) => p1.has_north() && p2.has_south(),
            (1, 0)  => p1.has_south() && p2.has_north(),
            (0, -1) => p1.has_west() && p2.has_east(),
            (0, 1)  => p1.has_east() && p2.has_west(),
            _ => false
        }
    }

    pub fn get_start(&self) -> AOCResult<(usize, usize)> {
        for (h, row) in self.map.iter().enumerate() {
            for (w, p) in row.iter().enumerate() {
                if *p == Pipe::Start {
                    return Ok((h, w));
                }
            }
        }
        Err(AOCError::ProcessingError("No start position found.".into()))
    }

    pub fn parse(input: impl AsRef<Path>) -> AOCResult<PipeMap> {
        let lines = read_lines_as_bytes(input)?;

        let map = lines
            .iter()
            .map(|line| {
                line.iter()
                    .map(|c| Pipe::from_char(*c as char))
                    .collect::<AOCResult<Vec<Pipe>>>()
            })
            .collect::<AOCResult<Vec<Vec<Pipe>>>>()?;

        Ok(PipeMap::new(map)?)
    }

}

#[derive(Clone, Debug)]
struct SearchPath {
    pub path: Vec<(usize, usize)>,
    pub visited: HashSet<(usize, usize)>,
}

impl SearchPath {
    pub fn new() -> Self {
        SearchPath { path: Vec::new(), visited: HashSet::new() }
    }

    pub fn add(&mut self, pos: (usize, usize)) {
        self.path.push(pos);
        self.visited.insert(pos);
    }

    pub fn has_visited(&self, pos: &(usize, usize)) -> bool {
        self.visited.contains(&pos)
    }
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let pipe_map = PipeMap::parse(input)?;
    let start_pos = pipe_map.get_start()?;

    pipe_map
        .get_enclosure_path(start_pos)
        .map(|path| (path.len() / 2).to_string())
        .ok_or_else(|| AOCError::ProcessingError("No Enclosure Found!".into()))
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {

    // Start with walking all the edges as possible start points - fill a stack of paths.
    // store visiting nodes as a tuple of 2 positions.
    // this code would be easier if I change to position types.
    // Being on a single space would be (p, p) where both positions are equal
    // Being squeezed between pipes would be (p1, p2)
    // I need to know on squeeze pipes what directions you can go.
    // Do the search in this way from the outside edges and fill all space.
    
    // Can you squeeze between pipes on an edge?
    // It might be better to use an enum of positions
    // enum Positions {
    //    Space(h, w),
    //    PipeSqueeze(h1, w1, h2, w2),
    //    EdgePipeSqueeze(h, w) 
    // }
    //
    // Once the outside positions are marked,
    // Scan through the board and look for Ground not marked.
    //
    // Answer in morning.


    Err(AOCError::ProcessingError("Not implemented".into()))
}