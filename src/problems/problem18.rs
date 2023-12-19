use std::collections::HashSet;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;


use lazy_static::lazy_static;
use regex::Regex;

use crate::aocbase::{AOCResult, AOCError};

lazy_static! {
    static ref DIG_OPERATION_REGEX: Regex = Regex::new(
        r"^\s*([UDLR])\s+(\d+)\s+\(#([0-9a-f]+)\)\s*$"
    ).unwrap();
}

#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Up = 0,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn apply_vertex(&self, amount: i32, vertex: &Vertex) -> Vertex {
        use Direction::*;
        match self {
            Up => Vertex::new(vertex.y - amount, vertex.x),
            Down => Vertex::new(vertex.y + amount, vertex.x),
            Left => Vertex::new(vertex.y, vertex.x - amount),
            Right => Vertex::new(vertex.y, vertex.x + amount),
        }
    }
}

impl TryFrom<char> for Direction {
    type Error = AOCError;

    fn try_from(value: char) -> AOCResult<Direction> {
        use Direction::*;
        Ok(match value {
            'U' => Up,
            'D' => Down,
            'L' => Left,
            'R' => Right,
            _ => {
                return Err(AOCError::ParseError(format!("Invalid character: {}", value)));
            }
        })
    }
}

#[derive(Debug)]
pub struct DigOperation {
    pub direction: Direction,
    pub amount: i32,
    pub color: String,
}

impl DigOperation {
    
    pub fn get_fix_from_color(&self) -> AOCResult<DigOperation> {
        use Direction::*;

        let s_len = self.color.len();

        if s_len < 2 {
            return Err(AOCError::ParseError(format!("Invalid color fix: {}", self.color)));
        }

        let n_amount = i32::from_str_radix(&self.color[0..s_len-1], 16)?;
        let d_l_char = self.color[s_len-1..s_len].bytes().nth(0).unwrap() as char;

        // 0 means R, 1 means D, 2 means L, and 3 means U.
        let n_direction = match d_l_char {
            '0' => Right,
            '1' => Down,
            '2' => Left,
            '3' => Up,
            _ => return Err(AOCError::InvalidRegexOperation(format!("Invalid direction number: {}", d_l_char))),
        };

        Ok(Self { direction: n_direction, amount: n_amount, color: self.color.clone() })
    }

    pub fn parse(line: impl AsRef<str>) -> AOCResult<DigOperation> {
        let line = line.as_ref();

        let cap = DIG_OPERATION_REGEX
            .captures(line)
            .ok_or_else(|| AOCError::ParseError(format!("Invalid dig operation: {}", line)))?;

        let direction: Direction = cap
            .get(1)
            .ok_or_else(|| AOCError::InvalidRegexOperation("Invalid capture group (1)".into()))?
            .as_str()
            .chars()
            .nth(0).unwrap()
            .try_into()?;

        let amount = cap
            .get(2)
            .ok_or_else(|| AOCError::InvalidRegexOperation("Invalid capture group (2)".into()))?
            .as_str()
            .parse::<i32>()?;

        let color = cap
            .get(3)
            .ok_or_else(|| AOCError::InvalidRegexOperation("Invalid capture group (3)".into()))?
            .as_str()
            .to_string();

        Ok(DigOperation { direction, amount, color })
    }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Vertex {
    pub y: i32,
    pub x: i32,
}

impl Vertex {
    pub fn new(y: i32, x: i32) -> Self {
        Self { y, x }
    }
}

// Assumes lines are horizontal or vertical only.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Line {
    start: Vertex,
    end: Vertex,
}

impl Line {

    pub fn new(start: Vertex, end: Vertex) -> Self {
        Self { start, end }
    }

    pub fn sign_vector(&self) -> Vertex {
        let y_sign = (self.end.y - self.start.y).signum();
        let x_sign = (self.end.x - self.start.x).signum();
        Vertex::new(y_sign, x_sign)
    }

    pub fn is_vertical(&self) -> bool {
        self.start.x == self.end.x
    }

    pub fn y_min(&self) -> i32 {
        self.start.y.min(self.end.y)
    }

    pub fn y_max(&self) -> i32 {
        self.start.y.max(self.end.y)
    }
}

pub struct DigSite {
    pub lines: Vec<Line>,
    pub position: Vertex,
}

impl DigSite {

    pub fn new() -> Self {
        Self { position: Vertex::new(0, 0), lines: Vec::new() }
    }

    pub fn dig(&mut self, dig_operation: &DigOperation) {
        let new_pos = dig_operation.direction.apply_vertex(dig_operation.amount, &self.position);
        self.lines.push(Line::new(self.position, new_pos));
        self.position = new_pos;
    }

    pub fn get_y_min(&self) -> i32 {
        self.lines.iter().map(|line| line.start.y.min(line.end.y)).min().unwrap_or(0)
    }
    
    pub fn get_y_max(&self) -> i32 {
        self.lines.iter().map(|line| line.start.y.max(line.end.y)).max().unwrap_or(0)
    }

    pub fn get_x_min(&self) -> i32 {
        self.lines.iter().map(|line| line.start.x.min(line.end.x)).min().unwrap_or(0)
    }
    
    pub fn get_x_max(&self) -> i32 {
        self.lines.iter().map(|line| line.start.x.max(line.end.x)).max().unwrap_or(0)
    }

    fn get_yx_vals(&self) -> (Vec<i32>, Vec<i32>) {
        let mut x_vals_hs: HashSet<i32> = HashSet::new();
        let mut y_vals_hs: HashSet<i32> = HashSet::new();
        
        for line in &self.lines {
            x_vals_hs.insert(line.start.x);
            x_vals_hs.insert(line.end.x);
            y_vals_hs.insert(line.start.y);
            y_vals_hs.insert(line.end.y);
        }

        let mut x_vals: Vec<i32> = Vec::with_capacity(x_vals_hs.len());
        x_vals.extend(x_vals_hs);
        x_vals.sort();

        let mut y_vals: Vec<i32> = Vec::with_capacity(y_vals_hs.len());
        y_vals.extend(y_vals_hs);
        y_vals.sort();

        (y_vals, x_vals)

    }

    fn find_vertical_line_overlap<'a>(&'a self, x: i32, y_start: i32, y_end: i32) -> Option<&'a Line> {
            // check starting lines
            for line in &self.lines {
                if line.is_vertical() &&
                    line.start.x == x &&
                    line.y_min() <= y_start &&
                    line.y_max() >= y_end
                {
                    return Some(line);
                }    
            }
            None
    }

    pub fn area(&self) -> i64 {
        let (y_vals, x_vals) = self.get_yx_vals();
    
        #[derive(Debug)]
        struct BoxInfo {
            x_idx: i32,
            y_idx: i32,
            width: i32,
            height: i32,
        }

        let mut inside_boxes: HashMap<(i32, i32), BoxInfo> = HashMap::new();

        // First find all the boxes and index them to find ones next to each other.
        for y_idx in 0 .. y_vals.len() - 1 {
            let y_start = y_vals[y_idx];
            let y_end = y_vals[y_idx + 1];

            let mut inside = false;

            for x_idx in 0 .. x_vals.len() - 1 {
                let x_start = x_vals[x_idx];
                let x_end = x_vals[x_idx + 1];

                let left_is_border = self.find_vertical_line_overlap(x_start, y_start, y_end).is_some();

                if left_is_border {
                    inside = !inside;
                }

                if inside {
                    inside_boxes.insert((y_idx as i32, x_idx as i32), BoxInfo {
                        y_idx: y_idx as i32, x_idx: x_idx as i32,
                        width: x_end - x_start + 1,
                        height: y_end - y_start + 1,
                    });
                }

            }
        }

        let mut total_area: i64 = 0;

        for inside_box in inside_boxes.values() {
            let mut total_box_area = inside_box.width as i64 * inside_box.height as i64;

            // This messy part is to determine if there are overlapping boxes and adjust area correctly.

            let opt_box_down = inside_boxes.get(&(inside_box.y_idx + 1, inside_box.x_idx));
            let opt_box_right = inside_boxes.get(&(inside_box.y_idx, inside_box.x_idx + 1));
            let opt_box_down_right = inside_boxes.get(&(inside_box.y_idx + 1, inside_box.x_idx + 1));
            let opt_box_down_left = inside_boxes.get(&(inside_box.y_idx + 1, inside_box.x_idx - 1));

            match (opt_box_down, opt_box_right, opt_box_down_right) {
                (Some(_), None, _)       => total_box_area -= inside_box.width as i64,
                (None, Some(_), _)       => total_box_area -= inside_box.height as i64,
                (Some(_), Some(_), _)    => total_box_area -= inside_box.width as i64 + inside_box.height as i64 - 1,
                (None, None, Some(_))    => total_box_area -= 1,
                (None, None, None)       => {}
            }

            match (opt_box_down_left, opt_box_down) {
                (Some(_), None) => total_box_area -= 1,
                _ => {}
            }

            total_area += total_box_area;
        }

        total_area
    }

    #[allow(dead_code)]
    pub fn render(&self) -> String {
        let y_min = self.get_y_min();
        let y_max = self.get_y_max();

        let x_min = self.get_x_min();
        let x_max = self.get_x_max();

        let mut output: Vec<Vec<char>> = Vec::new();

        // Draw empty
        for _ in y_min ..= y_max {
            let mut line: Vec<char> = Vec::new();
            for _ in x_min ..= x_max {
                line.push(' ');
            }
            output.push(line);
        }

        // Draw each line
        for line in &self.lines {
            let mut cur = line.start;
            let delta = line.sign_vector();
            loop {
                output[(cur.y - y_min) as usize][(cur.x - x_min) as usize] = '#';
                let next_cur = Vertex::new(cur.y + delta.y, cur.x + delta.x);
                if next_cur == line.end {
                    break;
                }
                cur = next_cur;
            }
        }

        // Turn to single string
        let mut s = String::new();

        for row in output {
            s.extend(row);
            s.push('\n');
        }

        s
    }
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let reader = BufReader::new(File::open(input)?);

    let mut dig_site = DigSite::new();

    for line in reader.lines() {
        let line = line?;
        let dig_op = DigOperation::parse(line)?;
        dig_site.dig(&dig_op);
    }

    let result = dig_site.area();

    Ok(result.to_string())
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    let reader = BufReader::new(File::open(input)?);

    let mut dig_site = DigSite::new();

    for line in reader.lines() {
        let line = line?;
        let dig_op = DigOperation::parse(line)?;
        let dig_op_fixed = dig_op.get_fix_from_color()?;
        dig_site.dig(&dig_op_fixed);
    }

    let result = dig_site.area();

    Ok(result.to_string())
}
