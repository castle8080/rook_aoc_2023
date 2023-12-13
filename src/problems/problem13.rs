use std::path::Path;
use std::mem::replace;
use std::collections::HashSet;

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

impl GroundCover {
    pub fn flip(&self) -> Self {
        match self {
            GroundCover::Ash => GroundCover::Rock,
            GroundCover::Rock => GroundCover::Ash,
        }
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

    #[allow(dead_code)]
    pub fn render(&self) -> String {
        let mut output = String::new();

        for row in &self.map {
            for val in row {
                output.push(match val {
                    GroundCover::Ash => '.',
                    GroundCover::Rock => '#',
                });
            }
            output.push('\n');
        }

        output
    }

    pub fn columns_equal(&self, c1: usize, c2: usize) -> bool {
        self.map.iter().all(|row| row[c1] == row[c2])
    }

    pub fn find_column_diffs(&self, c1: usize, c2: usize) -> Vec<usize> {
        self.map.iter()
            .enumerate()
            .filter(|(_, row)| row[c1] != row[c2])
            .map(|(r, _)| r)
            .collect()
    }

    pub fn rows_equal(&self, r1: usize, r2: usize) -> bool {
        self.map[r1] == self.map[r2]
    }

    pub fn find_row_diffs(&self, r1: usize, r2: usize) -> Vec<usize> {
        self.map[r1]
            .iter()
            .zip(&self.map[r2])
            .enumerate()
            .filter(|(_, (v1, v2))| v1 != v2)
            .map(|(c, _)| c)
            .collect()
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

    pub fn count_positions_in_vertical_mirror_view<'i>(
        &self, c: usize,
        items: impl Iterator<Item=&'i (usize, usize)>) -> usize
    {
        let dist = c.min((self.island_map.width() - 1) - (c + 1));
        let lower = c - dist;
        let upper = c + dist;
        //println!("count_positions_in_vertical_mirror_view: c={} lower={} upper={} dist={}", c, lower, upper, dist);
        items
            .filter(|(_, item_c)| item_c >= &lower && item_c <= &upper)
            .count()
    }

    pub fn count_positions_in_horizontal_mirror_view<'i>(
        &self, r: usize,
        items: impl Iterator<Item=&'i (usize, usize)>) -> usize
    {
        let dist = r.min((self.island_map.height() - 1) - (r + 1));
        let lower = r - dist;
        let upper = r + 1 + dist;
        items
            .filter(|(item_r, _)| item_r >= &lower && item_r <= &upper)
            .count()
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

    fn find_smudges(&self) -> HashSet<(usize, usize)> {
        let row_smudges = self.find_row_mirror_smudges();
        let col_smudges = self.find_column_mirror_smudges();

        let mut all_smudges: HashSet<(usize, usize)> = HashSet::new();
        all_smudges.extend(row_smudges);
        all_smudges.extend(col_smudges);

        all_smudges
    }

    fn find_row_mirror_smudges(&self) -> Vec<(usize, usize)> {
        (0 .. self.island_map.height() - 1)
            .flat_map(|row| self.find_row_mirror_smudge(row))
            .collect()
    }

    fn find_column_mirror_smudges(&self) -> Vec<(usize, usize)> {
        (0 .. self.island_map.width() - 1)
            .flat_map(|col| self.find_column_mirror_smudge(col))
            .collect()
    }

    fn find_row_mirror_smudge(&self, row: usize) -> Option<(usize, usize)> {
        let smudge = Self::find_smudge(row, self.island_map.height(), |r1, r2| {
            self.island_map.find_row_diffs(r1, r2)
        });

        smudge.map(|(r1, _r2, c)| (r1, c))
    }

    fn find_column_mirror_smudge(&self, col: usize) -> Option<(usize, usize)> {
        let smudge = Self::find_smudge(col, self.island_map.width(), |c1, c2| {
            self.island_map.find_column_diffs(c1, c2)
        });

        smudge.map(|(c1, _c2, r)| (r, c1))
    }

    fn find_smudge<F>(lower: usize, max: usize, f: F) -> Option<(usize, usize, usize)>
        where F: Fn(usize, usize) -> Vec<usize>
    {
        let dist = (lower + 1).min(max - lower - 1);
        let mut smudge: Option<(usize, usize, usize)> = None;

        for delta in 0 .. dist {
            let lower_pos = lower - delta;
            let upper_pos = lower + 1 + delta;
            let diff_positions = f(lower_pos, upper_pos);
            if diff_positions.len() > 1 {
                return None;
            }
            else if diff_positions.len() == 1 {
                if smudge.is_some() {
                    return None;
                }
                else {
                    smudge = Some((lower_pos, upper_pos, diff_positions[0]));
                }
            }
        }

        smudge
    }
}

pub fn score(verticals: &Vec<usize>, horizontals: &Vec<usize>) -> usize {
    verticals.iter().map(|v| *v + 1).sum::<usize>() +
        100 * horizontals.iter().map(|h| h + 1).sum::<usize>()
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let mut result: usize = 0;

    IslandMap::parse_each(input, |island_map| {
        let mirror_finder = MirrorFinder::new(&island_map);

        let verticals = mirror_finder.find_verticals();
        let horizontals = mirror_finder.find_horizontals();

        result += score(&verticals, &horizontals);
        Ok(())
    })?;

    Ok(result.to_string())
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    let mut result: usize = 0;

    IslandMap::parse_each(input, |mut island_map| {
        let mirror_finder = MirrorFinder::new(&island_map);

        // Fix the smudges!
        let smudges = mirror_finder.find_smudges();
        
        for (r, c) in &smudges {
            island_map.map[*r][*c] = island_map.map[*r][*c].flip();
        }
        
        // Need a new mirror finder after mutation.
        let mirror_finder = MirrorFinder::new(&island_map);

        // Each valid mirror had to have a smudge in it!
        let verticals = mirror_finder.find_verticals();
        let horizontals = mirror_finder.find_horizontals();


        let horizontals = horizontals
            .iter()
            .filter(|r| {
                let count = *(&mirror_finder.count_positions_in_horizontal_mirror_view(**r, smudges.iter()));
                count == 1
            })
            .map(|c| *c)
            .collect::<Vec<usize>>();

        let verticals = verticals
            .iter()
            .filter(|c| {
                let count = *(&mirror_finder.count_positions_in_vertical_mirror_view(**c, smudges.iter()));
                count == 1
            })
            .map(|c| *c)
            .collect::<Vec<usize>>();

        result += score(&verticals, &horizontals);

        Ok(())
    })?;

    Ok(result.to_string())
}