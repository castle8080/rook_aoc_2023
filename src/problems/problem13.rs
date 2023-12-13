use std::path::Path;
use std::mem::replace;

use crate::aocbase::{AOCResult, AOCError};
use crate::aocio::each_line;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum GroundCover {
    Rock = 0,
    Ash,
}

impl GroundCover {
    pub fn parse(c: char) -> AOCResult<Self> {
        Ok(match c {
            '.' => GroundCover::Ash,
            '#' => GroundCover::Rock,
            _ => {
                return Err(AOCError::ParseError(format!("Invalid ground cover: ({})", c)));
            }
        })
    }
}

#[derive(Debug)]
pub struct IslandMap {
    map: Vec<Vec<GroundCover>>,
}

impl IslandMap {

    pub fn new(map: Vec<Vec<GroundCover>>) -> Self {
        Self { map }
    }

    pub fn columns_equal(&self, c1: usize, c2: usize) -> bool {
        self.map.iter().all(|row| row[c1] == row[c2])
    }

    pub fn rows_equal(&self, r1: usize, r2: usize) -> bool {
        self.map[r1] == self.map[r2]
    }

    pub fn height(&self) -> usize {
        self.map.len()
    }

    pub fn width(&self) -> usize {
        self.map[0].len()
    }

    pub fn parse_each<F>(input: impl AsRef<Path>, mut f: F) -> AOCResult<()>
        where F: FnMut(IslandMap) -> AOCResult<()>
    {
        let mut map: Vec<Vec<GroundCover>> = Vec::new();

        each_line(input, |line| {
            let line = line.trim();
            if line.len() == 0 {
                if map.len() > 0 {
                    f(IslandMap::new(replace(&mut map, Vec::new())))?;
                }
            }
            else {
                let map_row = line
                    .chars()
                    .map(|c| GroundCover::parse(c))
                    .collect::<AOCResult<Vec<GroundCover>>>()?;
                map.push(map_row);
            }
            Ok(())
        })?;

        if map.len() > 0 {
            f(IslandMap::new(map))?;
        }

        Ok(())
    }
}

pub struct MirrorFinder<'a> {
    island_map: &'a IslandMap,
}

impl<'a> MirrorFinder<'a> {
    pub fn new(island_map: &'a IslandMap) -> Self {
        Self { island_map }
    }

    pub fn find_horizontals(&self) -> Vec<usize> {
        (0 .. self.island_map.height() - 1)
            .filter(|row| self.is_row_mirror(*row))
            .collect()
    }

    pub fn find_verticals(&self) -> Vec<usize> {
        (0 .. self.island_map.width() - 1)
            .filter(|col| self.is_column_mirror(*col))
            .collect()
    }

    fn is_row_mirror(&self, row: usize) -> bool {
        Self::is_mirror(row, self.island_map.height(), |u, l| {
            self.island_map.rows_equal(u, l)
        })
    }

    fn is_column_mirror(&self, col: usize) -> bool {
        Self::is_mirror(col, self.island_map.width(), |l, r| {
            self.island_map.columns_equal(l, r)
        })
    }

    fn is_mirror<F>(lower: usize, max: usize, f: F) -> bool
        where F: Fn(usize, usize) -> bool
    {
        let dist = (lower + 1).min(max - lower - 1);
        (0 .. dist).all(|delta| f(lower - delta, lower + 1 + delta))
    }
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let mut result: usize = 0;

    IslandMap::parse_each(input, |island_map| {
        let mirror_finder = MirrorFinder::new(&island_map);
        let verticals = mirror_finder.find_verticals();
        let horizontals = mirror_finder.find_horizontals();

        let map_result =
            verticals.iter().map(|v| *v + 1).sum::<usize>() +
            100 * horizontals.iter().map(|h| h + 1).sum::<usize>();

        result += map_result;

        Ok(())
    })?;

    Ok(result.to_string())
}