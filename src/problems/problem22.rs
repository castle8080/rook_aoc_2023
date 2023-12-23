use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;

use lazy_static::lazy_static;
use regex::Regex;

use crate::aocbase::{AOCResult, AOCError};
use crate::regex_ext::CapturesExt;
use crate::regex_ext::RegexExt;

lazy_static! {
    static ref PIECE_REGEX: Regex = Regex::new(
        r"^\s*(\d+),(\d+),(\d+)~(\d+),(\d+),(\d+)\s*$"
    ).unwrap();
}

#[derive(Debug, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

// This iterator moves towards an end
// It goes 1 space at a time going x, y, than z
// in straight lines.
pub struct PositionToItertor {
    cur: Position,
    end: Position,
    at_end: bool,
}

impl PositionToItertor {
    pub fn new(cur: Position, end: Position) -> Self {
        Self { cur, end, at_end: false }
    }
}

impl Iterator for PositionToItertor {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.at_end {
            return None;
        }

        let result = self.cur.clone();

        if self.cur.x < self.end.x {
            self.cur.x += 1;
        }
        else if self.cur.x > self.end.x {
            self.cur.x -= 1;
        }
        else if self.cur.y < self.end.y {
            self.cur.y += 1;
        }
        else if self.cur.y > self.end.y {
            self.cur.y -= 1;
        }
        else if self.cur.z < self.end.z {
            self.cur.z += 1;
        }
        else if self.cur.z > self.end.z {
            self.cur.z -= 1;
        }
        else {
            self.at_end = true;
        }

        Some(result)
    }
}

#[derive(Debug, Clone)]
pub struct Piece {
    pub id: i32,
    pub start: Position,
    pub end: Position,
}

impl Piece {

    pub fn position_iter(&self) -> PositionToItertor {
        PositionToItertor::new(self.start.clone(), self.end.clone())
    }

    pub fn parse(text: impl AsRef<str>) -> AOCResult<Piece> {
        let cap = PIECE_REGEX.captures_must(text.as_ref())?;

        let start = Position {
            x: cap.get_group(1)?.parse::<i32>()?,
            y: cap.get_group(2)?.parse::<i32>()?,
            z: cap.get_group(3)?.parse::<i32>()?,
        };

        let end = Position {
            x: cap.get_group(4)?.parse::<i32>()?,
            y: cap.get_group(5)?.parse::<i32>()?,
            z: cap.get_group(6)?.parse::<i32>()?,
        };

        Ok(Self { id: -1, start, end })
    }

    pub fn get_yx_lows(&self) -> HashMap<(i32, i32), i32> {
        let mut lows: HashMap<(i32, i32), i32> = HashMap::new();

        for pos in self.position_iter() {
            if let Some(yx_low) = lows.get_mut(&(pos.y, pos.x)) {
                if *yx_low > pos.z {
                    *yx_low = pos.z;
                }
            }
            else {
                lows.insert((pos.y, pos.x), pos.z);
            }
        }

        lows
    }

    pub fn get_low_z(&self) -> i32 {
        self.start.z.min(self.end.z)
    }
}

pub const EMPTY_PIECE_ID: i32 = -1;
pub const GROUND_ID: i32 = -2;

#[derive(Debug, Clone)]
pub struct Pieces {
    pub pieces: HashMap<i32, Piece>,

    // Represents the 3-d space with the value being the piece id.
    space_matrix: Vec<Vec<Vec<i32>>>,
}

impl Pieces {

    pub fn new(pieces: Vec<Piece>) -> AOCResult<Self> {

        let mut _pieces: HashMap<i32, Piece> = HashMap::new();
        for p in pieces {
            _pieces.insert(p.id, p);
        }

        let mut _self = Self { pieces: _pieces, space_matrix: Vec::new() };
        _self.intialize_space_matrix()?;
        Ok(_self)
    }

    fn intialize_space_matrix(&mut self) -> AOCResult<()> {

        let max_z = self.pieces
            .values()
            .flat_map(|p| vec![p.start.z, p.end.z])
            .max()
            .ok_or_else(|| AOCError::ProcessingError(format!("Invalid pieces.")))?;

        let max_y = self.pieces
            .values()
            .flat_map(|p| vec![p.start.y, p.end.y])
            .max()
            .ok_or_else(|| AOCError::ProcessingError(format!("Invalid pieces.")))?;

        let max_x = self.pieces
            .values()
            .flat_map(|p| vec![p.start.x, p.end.x])
            .max()
            .ok_or_else(|| AOCError::ProcessingError(format!("Invalid pieces.")))?;

        // matrix will be y, x, z
        // Initialize the empty space

        for _y in 0 ..= max_y {
            let mut plane: Vec<Vec<i32>> = Vec::new();
            for _x in 0 ..= max_x {
                plane.push(vec![EMPTY_PIECE_ID; max_z as usize + 1]);
            }
            self.space_matrix.push(plane);
        }

        // Set space ids

        let space_matrix = &mut self.space_matrix;

        for piece in self.pieces.values() {
            for pos in piece.position_iter() {
                if space_matrix[pos.y as usize][pos.x as usize][pos.z as usize] != EMPTY_PIECE_ID {
                    return Err(AOCError::ProcessingError(format!("Too many things in a space.")));
                }
                else {
                    space_matrix[pos.y as usize][pos.x as usize][pos.z as usize] = piece.id;
                }
            }
        }

        Ok(())
    }

    pub fn disintegrate(&mut self, piece_id: i32) {
        if let Some(p) = self.pieces.remove(&piece_id) {
            for pos in p.position_iter() {
                self.space_matrix[pos.y as usize][pos.x as usize][pos.z as usize] = EMPTY_PIECE_ID;
            }
        }
    }

    // Utility for inspecting the space matrix
    #[allow(dead_code)]
    pub fn get_space_matrix_debug_info(&self) -> String {
        let mut output = String::new();

        let mut fill_count = 0;

        for z in (0 .. self.space_matrix[0][0].len()).rev() {
            output.push_str(format!("Layer: {}", z).as_str());
            output.push('\n');
            for y in 0 .. self.space_matrix.len() {
                for x in 0 .. self.space_matrix[0].len() {
                    let p_id = self.space_matrix[y][x][z];
                    if p_id != EMPTY_PIECE_ID {
                        output.push_str(format!("  * [{},{}] -> {}", y, x, p_id).as_str());
                        output.push('\n');
                        fill_count += 1;
                    }
                }
            }
        }

        output.push_str(format!("Fill Count: {}", fill_count).as_str());
        output.push('\n');

        output
    }

    pub fn get_disintegratable(&self) -> Vec<i32> {
        let held_by = self.get_held_by();

        let mut disintegratable: Vec<i32> = Vec::new();

        // See if any are not solely supporting
        for p in self.pieces.values() {
            let mut support_count = 0;

            for (_, supporting_ids) in &held_by {
                if supporting_ids.contains(&p.id) {
                    if supporting_ids.len() == 1 {
                        support_count += 1;
                    }
                }
            }

            if support_count == 0 {
                disintegratable.push(p.id);
            }
        }

        disintegratable
    }

    fn get_held_by(&self) -> HashMap<i32, HashSet<i32>> {
        // Start by building up maps to know for each piece what is holding it up.
        // This could be the ground.

        let mut held_by: HashMap<i32, HashSet<i32>> = HashMap::new();

        for p in self.pieces.values() {
            let mut p_held_by: HashSet<i32> = HashSet::new();

            for pos in p.position_iter() {
                if pos.z == 1 {
                    p_held_by.clear();
                    p_held_by.insert(GROUND_ID);
                }
                else {
                    let other_id = self.space_matrix[pos.y as usize][pos.x as usize][(pos.z - 1) as usize];
                    if other_id != p.id && other_id != EMPTY_PIECE_ID {
                        p_held_by.insert(other_id);
                    }
                }
            }

            held_by.insert(p.id, p_held_by);
        }

        held_by
    }

    pub fn lower(&mut self) -> i32 {
        let mut potential_lowerable_pieces = self
            .pieces
            .values()
            .filter(|p| p.get_low_z() > 1)
            .map(|p| (p.get_low_z(), p.id))
            .collect::<Vec<(i32, i32)>>();

        potential_lowerable_pieces
            .sort_by_key(|(low_z, _p_id)| *low_z);

        let mut lower_count = 0;

        for (_, id) in &potential_lowerable_pieces {
            if self.lower_piece(*id) {
                lower_count += 1;
            }
        }

        return lower_count;
    }

    fn lower_piece(&mut self, piece_id: i32) -> bool {
        let p = self.pieces.get(&piece_id).unwrap();

        let yx_lows = p.get_yx_lows();

        let mut z_deltas: Vec<i32> = Vec::new();

        // Figure out what can lower.
        for ((y, x), low_z) in yx_lows {

            let mut z_delta = 0;
            let z_col = &self.space_matrix[y as usize][x as usize];

            while low_z - z_delta > 1 {
                let next_space_p_id = z_col[(low_z - z_delta - 1) as usize];
                if next_space_p_id != EMPTY_PIECE_ID {
                    break;
                }
                else {
                    z_delta += 1;
                }
            }

            z_deltas.push(z_delta);
        }

        // Get the min of z deltas
        match z_deltas.iter().min() {
            Some(z_delta) if *z_delta > 0 => {
                self.move_piece_down(piece_id, *z_delta);
                true
            },
            _ => {
                false
            }
        }
    }

    fn move_piece_down(&mut self, piece_id: i32, z_delta: i32) {
        if z_delta <= 0 {
            return;
        }

        let p = &mut self.pieces.get_mut(&piece_id).unwrap();

        // Move the space ids in the space matrix first
        for pos in p.position_iter() {
            let new_z = pos.z - z_delta;

            let prev_id = self.space_matrix[pos.y as usize][pos.x as usize][new_z as usize];

            self.space_matrix[pos.y as usize][pos.x as usize][new_z as usize] = p.id;

            // This swap is to account for not caring about order of moving an object down.
            // This method assumes the move has been validated.
            self.space_matrix[pos.y as usize][pos.x as usize][pos.z as usize] = prev_id;
        }

        // Mutate the piece itself too
        p.start.z -= z_delta;
        p.end.z -= z_delta;
    }

    pub fn parse(input: impl AsRef<Path>) -> AOCResult<Self> {
        let reader = BufReader::new(File::open(input)?);
        let mut pieces: Vec<Piece> = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.len() > 0 {
                let mut piece = Piece::parse(line)?;
                piece.id = pieces.len() as i32 + 1;
                pieces.push(piece);
            }

        }

        Self::new(pieces)
    }

}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let mut pieces = Pieces::parse(input)?;
    pieces.lower();

    let disentegratable = pieces.get_disintegratable();
    let result = disentegratable.len();

    Ok(result.to_string())
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    let mut pieces = Pieces::parse(input)?;
    pieces.lower();

    let mut total_affect_count: i32 = 0;

    for piece in pieces.pieces.values() {
        let mut pieces_new = pieces.clone();
        pieces_new.disintegrate(piece.id);
        let lower_count = pieces_new.lower();
        total_affect_count += lower_count;

    }

    Ok(total_affect_count.to_string())
}