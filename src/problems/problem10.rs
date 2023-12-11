use std::path::Path;
use std::collections::HashSet;

use crate::aocbase::{AOCResult, AOCError};
use crate::aocio::read_lines_as_bytes;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
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

    pub fn is_start(&self) -> bool {
        match self {
            Pipe::Start => true,
            _ => false,
        }
    }

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

    pub fn render_unicode(&self) -> &str {
        use Pipe::*;
        match self {
            NorthWest => "\u{2518}",
            NorthEast => "\u{2514}",
            SouthEast => "\u{250c}",
            SouthWest => "\u{2510}",
            NorthSouth => "\u{2502}",
            EastWest => "\u{2500}",
            Ground => " ",
            Start => "S",
        }
    }

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

    pub fn render(&self) -> String {
        let mut output = String::new();

        for line in self.map.iter() {
            for cell in line {
                output.push_str(cell.render_unicode());
            }
            output.push_str("\n");
        }

        output

    }

    pub fn width(&self) -> usize {
        self.map[0].len()
    }

    pub fn height(&self) -> usize {
        self.map.len()
    }

    fn get_connected_positions(&self, (h, w): (usize, usize)) -> Vec<(usize, usize)> {
        let mut connections: Vec<(usize, usize)> = Vec::new();

        let max_h = self.height();
        let max_w = self.width();

        if h > 0  && self.is_connected((h, w), (h-1, w)) {
            connections.push((h-1, w));
        }
        if h < max_h - 1 && self.is_connected((h, w), (h+1, w)) {
            connections.push((h+1, w));
        }
        if w > 0 && self.is_connected((h, w), (h, w-1)) {
            connections.push((h, w-1));
        }
        if w < max_w - 1 && self.is_connected((h, w), (h, w+1)) {
            connections.push((h, w+1));
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
                //println!("connected: {next_h}, {next_w}");
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
            // 2 above 1
            (-1, 0) => p1.has_north() && p2.has_south(),
            // 2 below 1
            (1, 0)  => p1.has_south() && p2.has_north(),
            // 2 left of 1
            (0, -1) => p1.has_west() && p2.has_east(),
            // 2 right of 1
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

    //println!("Solving for map:\n{}", pipe_map.render());

    pipe_map
        .get_enclosure_path(start_pos)
        .map(|path| (path.len() / 2).to_string())
        .ok_or_else(|| AOCError::ProcessingError("No Enclosure Found!".into()))
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Corner {
    UpperLeft = 0,
    UpperRight,
    LowerLeft,
    LowerRight,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct SpaceCorner {
    h: usize,
    w: usize,
    corner: Corner,
}

struct InnerSpaceSolver<'a> {
    pipe_map: &'a PipeMap,
    visited: HashSet<SpaceCorner>,
    search_stack: Vec<SpaceCorner>,
    outside_locations: HashSet<(usize, usize)>,
    enclosure_locations: HashSet<(usize, usize)>,
}

impl<'a> InnerSpaceSolver<'a> {

    pub fn new(pipe_map: &'a PipeMap, enclosure_path: &Vec<(usize, usize)>) -> Self {
        let mut enclosure_locations: HashSet<(usize, usize)> = HashSet::new();
        for pos in enclosure_path {
            enclosure_locations.insert(*pos);
        }

        InnerSpaceSolver {
            pipe_map,
            visited: HashSet::new(),
            search_stack: Vec::new(),
            outside_locations: HashSet::new(),
            enclosure_locations,
        }
    }

    pub fn solve(&mut self) -> i32 {
        self.clear();
        self.seed_search_stack();
        self.run_search();
        self.get_inner_tile_counts()

    }

    fn get_inner_tile_counts(&self) -> i32 {
        let max_h = self.pipe_map.height();
        let max_w = self.pipe_map.width();

        let mut inner_tile_count = 0;

        for h in 0..max_h {
            for w in 0..max_w {
                if !self.outside_locations.contains(&(h, w)) &&
                    !self.enclosure_locations.contains(&(h, w))
                {
                    inner_tile_count += 1;
                }
            }
        }

        inner_tile_count
    }

    fn run_search(&mut self) {
        while let Some(pos) = self.search_stack.pop() {
            self.on_visit(&pos);
            self.add_connections_to_visit(&pos);
        }
    }

    fn on_visit(&mut self, pos: &SpaceCorner) {
        // Track things of actual interest here
        self.outside_locations.insert((pos.h, pos.w));
    }

    fn clear(&mut self) {
        self.visited.clear();
        self.search_stack.clear();
        self.outside_locations.clear();
    }

    fn add_connections_to_visit(&mut self, pos: &SpaceCorner) {
        let max_h = self.pipe_map.height();
        let max_w = self.pipe_map.width();

        let pipe = self.pipe_map.map[pos.h][pos.w];

        match pos.corner {
            Corner::UpperLeft => {
                // Check upwards
                if pos.h > 0 && !self.pipe_map.map[pos.h - 1][pos.w].is_start() {
                    self.add_to_visit(SpaceCorner { h: pos.h - 1, w: pos.w, corner: Corner::LowerLeft });
                }
                // Check downwards
                if !pipe.has_west() {
                    self.add_to_visit(SpaceCorner { h: pos.h, w: pos.w, corner: Corner::LowerLeft });
                }
                // Check left
                if pos.w > 0 && !self.pipe_map.map[pos.h][pos.w - 1].is_start() {
                    self.add_to_visit(SpaceCorner { h: pos.h, w: pos.w - 1, corner: Corner::UpperRight });
                }
                // Check right
                if !pipe.has_north() {
                    self.add_to_visit(SpaceCorner { h: pos.h, w: pos.w, corner: Corner::UpperRight });
                }
            },
            Corner::UpperRight => {
                // Check upwards
                if pos.h > 0 && !self.pipe_map.map[pos.h - 1][pos.w].is_start() {
                    self.add_to_visit(SpaceCorner { h: pos.h - 1, w: pos.w, corner: Corner::LowerRight });
                }
                // Check downwards
                if !pipe.has_east() {
                    self.add_to_visit(SpaceCorner { h: pos.h, w: pos.w, corner: Corner::LowerRight });
                }
                // Check left
                if !pipe.has_north() {
                    self.add_to_visit(SpaceCorner { h: pos.h, w: pos.w, corner: Corner::UpperLeft });
                }
                // Check right
                if pos.w < max_w - 1 && !self.pipe_map.map[pos.h][pos.w + 1].is_start() {
                    self.add_to_visit(SpaceCorner { h: pos.h, w: pos.w + 1, corner: Corner::UpperLeft });
                }
            },
            Corner::LowerLeft => {
                // Check upwards
                if !pipe.has_west() {
                    self.add_to_visit(SpaceCorner { h: pos.h, w: pos.w, corner: Corner::UpperLeft });
                }
                // Check downwards
                if pos.h < max_h - 1 && !self.pipe_map.map[pos.h + 1][pos.w].is_start() {
                    self.add_to_visit(SpaceCorner { h: pos.h + 1, w: pos.w, corner: Corner::UpperLeft });
                }
                // Check left
                if pos.w > 0 && !self.pipe_map.map[pos.h][pos.w - 1].is_start() {
                    self.add_to_visit(SpaceCorner { h: pos.h, w: pos.w - 1, corner: Corner::LowerRight });
                }
                // Check right
                if !pipe.has_south() {
                    self.add_to_visit(SpaceCorner { h: pos.h, w: pos.w, corner: Corner::LowerRight });
                }
            },
            Corner::LowerRight => {
                // Check upwards
                if !pipe.has_east() {
                    self.add_to_visit(SpaceCorner { h: pos.h, w: pos.w, corner: Corner::UpperRight });
                }
                // Check downwards
                if pos.h < max_h - 1 && !self.pipe_map.map[pos.h + 1][pos.w].is_start() {
                    self.add_to_visit(SpaceCorner { h: pos.h + 1, w: pos.w, corner: Corner::UpperRight });
                }
                // Check left
                if !pipe.has_south() {
                    self.add_to_visit(SpaceCorner { h: pos.h, w: pos.w, corner: Corner::LowerLeft });
                }
                // Check right
                if pos.w < max_w - 1 && !self.pipe_map.map[pos.h][pos.w + 1].is_start() {
                    self.add_to_visit(SpaceCorner { h: pos.h, w: pos.w + 1, corner: Corner::LowerLeft });
                }
            }
        }
    }

    fn add_to_visit(&mut self, node: SpaceCorner) {
        if !self.visited.contains(&node) {
            self.search_stack.push(node.clone());
            self.visited.insert(node);
        }
    }

    fn seed_search_stack_single(&mut self, h: usize, w: usize) {
        let pipe = self.pipe_map.map[h][w];

        if pipe.is_start() {
            // Skip start for now, we don't know what it really is
            return;
        }

        let max_h = self.pipe_map.height();
        let max_w = self.pipe_map.width();

        if h == 0 {
            self.add_to_visit(SpaceCorner { h, w, corner: Corner::UpperLeft });
            self.add_to_visit(SpaceCorner { h, w, corner: Corner::UpperRight });
        }
        else if h == max_h - 1 {
            self.add_to_visit(SpaceCorner { h, w, corner: Corner::LowerLeft });
            self.add_to_visit(SpaceCorner { h, w, corner: Corner::LowerRight });
        }

        if w == 0 {
            self.add_to_visit(SpaceCorner { h, w, corner: Corner::UpperLeft });
            self.add_to_visit(SpaceCorner { h, w, corner: Corner::LowerLeft });
        }
        else if w == max_w - 1 {
            self.add_to_visit(SpaceCorner { h, w, corner: Corner::UpperRight });
            self.add_to_visit(SpaceCorner { h, w, corner: Corner::LowerRight });
        }
    }

    fn seed_search_stack(&mut self) {
        let max_h = self.pipe_map.height();
        let max_w = self.pipe_map.width();

        for w in 0 .. max_w {
            self.seed_search_stack_single(0, w);
            self.seed_search_stack_single(max_h - 1, w);
        }

        for h in 0 .. max_h - 1 {
            self.seed_search_stack_single(h, 0);
            self.seed_search_stack_single(h, max_w - 1);
        }
    }
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    let pipe_map = PipeMap::parse(input)?;
    let start_pos = pipe_map.get_start()?;

    //println!("Solving for map:\n{}", pipe_map.render());

    let enclosing_path = pipe_map
        .get_enclosure_path(start_pos)
        .ok_or_else(|| AOCError::ProcessingError("Could not find enclosing path.".into()))?;

    let mut ispace_solver = InnerSpaceSolver::new(&pipe_map, &enclosing_path);
    let result = ispace_solver.solve();

    Ok(result.to_string())
}