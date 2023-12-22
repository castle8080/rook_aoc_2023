use std::collections::HashMap;
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GardenVisitNode {
    pub y: i32,
    pub x: i32,
    pub even_odd: i32,
}

impl GardenVisitNode {

    pub fn new(y: i32, x: i32, even_odd: i32) -> Self {
        Self { y, x, even_odd: even_odd % 2 }
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
        let explored = self.explore(start_y, start_x);

        let tgt_even_odd = steps % 2;

        explored
            .iter()
            .filter(|(gv_node, tgt_steps)| gv_node.even_odd == tgt_even_odd && **tgt_steps <= steps)
            .map(|(gv_node, _)| (gv_node.y, gv_node.x))
            .collect()
    }

    pub fn explore(&self, start_y: i32, start_x: i32) -> HashMap<GardenVisitNode, i32>  {

        let mut x_queue: VecDeque<(i32, i32, i32)> = VecDeque::new();
        let mut visited: HashMap<GardenVisitNode, i32> = HashMap::new();

        match self.get(start_y, start_x) {
            None|Some(Space::Rock) => return visited,
            _ => {}
        }

        x_queue.push_back((start_y, start_x, 0));

        while let Some((cur_y, cur_x, cur_steps)) = x_queue.pop_front() {
            for (yd, xd) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let adj_steps = cur_steps + 1;

                let adj_gv_node = GardenVisitNode::new(
                    cur_y + yd,
                    cur_x + xd,
                    adj_steps % 2
                );

                match self.get(adj_gv_node.y, adj_gv_node.x) {
                    Some(Space::Start|Space::Plot) => {
                        if !visited.contains_key(&adj_gv_node) {
                            x_queue.push_back((adj_gv_node.y, adj_gv_node.x, adj_steps));
                            visited.insert(adj_gv_node, adj_steps);
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

struct InfiniteGardenPathSolver<'a> {
    pub garden: &'a Garden,
    pub max_steps: i32,
    pub debug: bool,
}

/*
 * This Solver is highly dependent on the data input and makes assumptions such as.
 * 1. The input is a square
 * 2. The start is in the center.
 * 3. The width and height is an odd amount.
 * 4. There are straight lines with no stones from center to each edge.
 * 5. There are borders with no stones.
 * 
 * This code would not work on a general repeating space.
 * 
 */
impl<'a> InfiniteGardenPathSolver<'a> {

    pub fn new(garden: &'a Garden, max_steps: i32, debug: bool) -> Self {
        Self { garden, max_steps, debug }
    }

    fn count_visits(visits: &HashMap<GardenVisitNode, i32>, steps: i32) -> i32 {
        let steps_even_odd = steps % 2;
        let mut count = 0;

        for (_, node_steps) in visits {
            if node_steps % 2 == steps_even_odd && *node_steps <= steps {
                count += 1;
            }
        }

        count
    }

    pub fn solve(&self) -> AOCResult<i128> {
        let (start_y, start_x) = self.garden.find_start()?;

        // This code could be broken down into seaparate methods, but
        // I am not sure it would make it any easier to read.
        // The calculations are very linear, and although long and tedious,
        // I find it easier to validate going straight down the code then jumping around
        // with mathy stuff like this.

        // How many boxes after the current can we go in any straight direction?
        let box_dist = (self.max_steps - start_x) as f64 / self.garden.height() as f64;
        if box_dist.floor() != box_dist {
            return Err(AOCError::ProcessingError("Can't deal with this!".into()));
        }
        let box_dist = box_dist as i32;

        // How much we have to move by to go 1/2 of a box distance.
        // If you are in the middle you should end up in the next box.
        let move_half_amt = self.garden.width() / 2 + 1;

        if self.debug {
            println!("Map: {} by {}", self.garden.height(), self.garden.width());
            println!("Start: {} by {}", start_y, start_x);
            println!("Is initial even/odd: {}", self.max_steps % 2);
            println!("Box dist: {box_dist}");
            println!("move_half_amt: {}", move_half_amt);
        }

        // Record visits from start
        let base_visits   = self.garden.explore(start_y, start_x);

        // Record visits from middle of edges
        let left_visits   = self.garden.explore(start_y, self.garden.width() - 1);
        let right_visits  = self.garden.explore(start_y, 0);
        let top_visits    = self.garden.explore(self.garden.height() - 1, start_x);
        let bottom_visits = self.garden.explore(0, start_x);

        // Record vsits from corners
        let tl_visits     = self.garden.explore(self.garden.height() - 1, self.garden.width() - 1);
        let tr_visits     = self.garden.explore(self.garden.height() - 1, 0);
        let bl_visits     = self.garden.explore(0, self.garden.width() - 1);
        let br_visits     = self.garden.explore(0, 0);

        // Count for full box at the starting point
        let base_count = Self::count_visits(&base_visits, self.max_steps) as i128;

        // What the full box count would be if you were on an alternate step
        let base_alt_count = Self::count_visits(&base_visits, self.max_steps - 1) as i128;

        if self.debug {
            println!("base_count:     {}", base_count);
            println!("base_alt_count: {}", base_alt_count);
        }

        // how many steps are left when you get past the edge of the last box in any direction.
        let end_steps_left = (self.max_steps - move_half_amt) % self.garden.width();

        // If you go to a diagnoal box with a short line how many left?
        let short_diagonal_end_steps_left = end_steps_left - move_half_amt;

        // Move over to the diagnoal leaving more space and see how many steps needed.
        let long_diagonal_end_steps_left = short_diagonal_end_steps_left + self.garden.width();

        if self.debug {
            println!("end_steps_left:                {}", end_steps_left);
            println!("short_diagonal_end_steps_left: {}", short_diagonal_end_steps_left);
            println!("long_diagonal_end_steps_left:  {}", long_diagonal_end_steps_left);
        }

        // Get the counts for the ends of the structure.

        let left_count   = Self::count_visits(&left_visits, end_steps_left) as i128;
        let right_count  = Self::count_visits(&right_visits, end_steps_left) as i128;
        let top_count    = Self::count_visits(&top_visits, end_steps_left) as i128;
        let bottom_count = Self::count_visits(&bottom_visits, end_steps_left) as i128;

        if self.debug {
            println!("left_count:   {}", left_count);
            println!("right_count:  {}", right_count);
            println!("top_count:    {}", top_count);
            println!("bottom_count: {}", bottom_count);
        }

        // Get the counts for the diagoanls
        let tl_short_count = Self::count_visits(&tl_visits, short_diagonal_end_steps_left) as i128;
        let tl_long_count = Self::count_visits(&tl_visits, long_diagonal_end_steps_left) as i128;

        let tr_short_count = Self::count_visits(&tr_visits, short_diagonal_end_steps_left) as i128;
        let tr_long_count = Self::count_visits(&tr_visits, long_diagonal_end_steps_left) as i128;
        
        let bl_short_count = Self::count_visits(&bl_visits, short_diagonal_end_steps_left) as i128;
        let bl_long_count = Self::count_visits(&bl_visits, long_diagonal_end_steps_left) as i128;
        
        let br_short_count = Self::count_visits(&br_visits, short_diagonal_end_steps_left) as i128;
        let br_long_count = Self::count_visits(&br_visits, long_diagonal_end_steps_left) as i128;

        if self.debug {
            println!("tl_short_count: {}", tl_short_count);
            println!("tl_long_count:  {}", tl_long_count);
            println!("tr_short_count: {}", tr_short_count);
            println!("tr_long_count:  {}", tr_long_count);
            println!("bl_short_count: {}", bl_short_count);
            println!("bl_long_count:  {}", bl_long_count);
            println!("br_short_count: {}", br_short_count);
            println!("br_long_count:  {}", br_long_count);
        }

        // Now that we have counts for box types, need to figure out how many of each box type

        // The area will be divided up into sections
        //   1. The middle base box (already done)
        //   2. The full boxes along a straing line from first box (lines)
        //   3. The ends (not full boxes)
        //   4. Full boxes in a quadrant
        //   5. Diagonal boxes


        // 2. full boxes along straight lines
        //    They should start with alternate counts
        let box_sl_count = (box_dist - 1) as i128;
        let box_sl_base_count = (box_sl_count / 2) as i128;
        let box_sl_alt_count = (box_sl_base_count + box_sl_count % 2) as i128;
        
        let sl_all_total = 4 * (box_sl_base_count * base_count as i128 + box_sl_alt_count * base_alt_count as i128);
        
        if self.debug {
            println!("box_sl_base_count: {}", box_sl_base_count); 
            println!("box_sl_alt_count:  {}", box_sl_alt_count);
            println!("sl_all_total:      {}", sl_all_total);
        }

        // 3. End count
        //    The work was mostly done and just need to add up.
        let ends_total = left_count + right_count + top_count + bottom_count;

        if self.debug {
            println!("ends_total: {}", ends_total);
        }

        // 4. Full boxes in a quadrant
        //    The work was mostly done and just need to add up.

        let mut full_box_quadrant_total: i128 = 0;

        // Yes there is math I could probably do to avoid this loop.
        for n in (1 ..= box_dist - 2).rev() {
            if n % 2 == 1 {
                full_box_quadrant_total += base_count * (n as i128);
            }
            else {
                full_box_quadrant_total += base_alt_count * (n as i128);
            }
        }

        let all_full_box_quandrants = full_box_quadrant_total * 4;

        if self.debug {
            println!("full_box_quadrant_total: {}", full_box_quadrant_total);
            println!("all_full_box_quandrants: {}", all_full_box_quandrants);
        }

        // 5. Diagonals
        //    There should be N pairs of them

        let box_short_count = box_dist as i128;
        let box_long_count = (box_dist - 1) as i128;
  
        let tl_diag_total = box_short_count * tl_short_count + box_long_count * tl_long_count;
        let tr_diag_total = box_short_count * tr_short_count + box_long_count * tr_long_count;
        let bl_diag_total = box_short_count * bl_short_count + box_long_count * bl_long_count;
        let br_diag_total = box_short_count * br_short_count + box_long_count * br_long_count;

        let diag_totals = tl_diag_total + tr_diag_total + bl_diag_total + br_diag_total;

        if self.debug {
            println!("tl_diag_total: {}", tl_diag_total);
            println!("tr_diag_total: {}", tr_diag_total);
            println!("bl_diag_total: {}", bl_diag_total);
            println!("br_diag_total: {}", br_diag_total);
            println!("diag_totals:   {}", diag_totals);
        }

        // 6 Final Total:

        let final_total =
            base_count +
            sl_all_total +
            ends_total +
            all_full_box_quandrants +
            diag_totals;

        if self.debug {
            println!(" Final Total: {}", final_total);
            println!("   * Middle Count:                {}", base_count);
            println!("   * Straight Line Count:         {}", sl_all_total);
            println!("   * Ends Count:                  {}", ends_total);
            println!("   * Full Boxes Quandrants Count: {}", all_full_box_quandrants);
            println!("   * Diagonals Count:             {}", diag_totals);
        }

        Ok(final_total)
    }
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    let garden = Garden::parse(input)?;
    let infinite_garden_solver = InfiniteGardenPathSolver::new(&garden, 26501365, false);

    let result = infinite_garden_solver.solve()?;

    Ok(result.to_string())
}