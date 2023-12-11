use std::path::Path;
use std::collections::HashMap;

use crate::aocbase::{AOCResult, AOCError};
use crate::aocio::read_lines_as_bytes;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum SpaceArea {
    Empty,
    Galaxy(u32),
}

#[derive(Debug)]
pub struct SpaceMap {
    pub galaxy_index: HashMap<u32, (usize, usize)>,
    pub width: usize,
    pub height: usize,
}

impl SpaceMap {

    pub fn from(map: &Vec<Vec<SpaceArea>>) -> Self {
        let height = map.len();
        let width = map[0].len();
        let galaxy_index = SpaceMap::get_galaxy_index(&map);

        SpaceMap {
            //map,
            width,
            height,
            galaxy_index,
        }
    }

    #[allow(dead_code)]
    fn get_reverse_galaxy_index(&self) -> HashMap<(usize, usize), u32> {
        let mut r_index: HashMap<(usize, usize), u32> = HashMap::new();
        for (id, (h, w)) in &self.galaxy_index {
            r_index.insert((*h, *w), *id);
        }
        r_index
    }

    #[allow(dead_code)]
    pub fn render_csv(&self) -> String {
        let mut output = String::new();
        let r_index = self.get_reverse_galaxy_index();

        for h in 0..self.height {
            for w in 0..self.width {
                output.push(',');
                if let Some(id) = r_index.get(&(h, w)) {
                    output.push_str(id.to_string().as_str());
                }
                else {
                    output.push(' ')
                }
            }
            output.push('\n');
        }
        output
    }

    #[allow(dead_code)]
    pub fn render(&self) -> String {
        let mut output = String::new();
        let r_index = self.get_reverse_galaxy_index();

        for h in 0..self.height {
            for w in 0..self.width {
                if r_index.contains_key(&(h, w)) {
                    output.push('*');
                }
                else {
                    output.push(' ')
                }
            }
            output.push('\n');
        }
        output
    }

    fn get_galaxy_index(map: &Vec<Vec<SpaceArea>>) -> HashMap<u32, (usize, usize)> {
        let mut galaxy_index: HashMap<u32, (usize, usize)> = HashMap::new();
        let width = map[0].len();
        let height = map.len();
        for h in 0 .. height {
            for w in 0 .. width {
                if let SpaceArea::Galaxy(id) = map[h][w] {
                    galaxy_index.insert(id, (h, w));
                }
            }
        }
        galaxy_index
    }

    pub fn calculate_galaxy_distances(&self) -> Vec<(u32, u32, usize)> {
        let mut distances: Vec<(u32, u32, usize)> = Vec::new();

        for (id1, (h1, w1)) in &self.galaxy_index {
            for (id2, (h2, w2)) in &self.galaxy_index {
                if id1 < id2 {
                    let dist = h1.abs_diff(*h2) + w1.abs_diff(*w2);
                    distances.push((*id1, *id2, dist));
                }
            }
        }

        distances
    }

    pub fn expand(&self, expand_amount: usize) -> SpaceMap {
        let empty_rows = self.get_empty_rows();
        let empty_columns = self.get_empty_columns();

        let new_width = self.width + empty_columns.len() * expand_amount;
        let new_height = self.height + empty_rows.len() * expand_amount;

        let mut col_map: Vec<usize> = Vec::new();
        {
            let mut width_expansion_count = 0;
            for i in 0..self.width {
                col_map.push(i + width_expansion_count);
                if empty_columns.contains(&i) {
                    width_expansion_count += expand_amount;
                }
            }
        }

        let mut row_map: Vec<usize> = Vec::new();
        {
            let mut height_expansion_count = 0;
            for i in 0..self.height {
                row_map.push(i + height_expansion_count);
                if empty_rows.contains(&i) {
                    height_expansion_count += expand_amount;
                }
            }
        }

        let mut new_galaxy_index: HashMap<u32, (usize, usize)> = HashMap::new();
        {
            for (id, (h, w)) in &self.galaxy_index {
                new_galaxy_index.insert(*id, (row_map[*h], col_map[*w]));
            }
        }

        SpaceMap {
            galaxy_index: new_galaxy_index,
            width: new_width,
            height: new_height,
        }
    }

    fn get_empty_columns(&self) -> Vec<usize> {
        let width = self.width;

        let mut markers = vec![true; width];
        for (_, w) in self.galaxy_index.values() {
            markers[*w] = false;
        }

        markers
            .iter()
            .enumerate()
            .filter(|(_, marked)| **marked)
            .map(|(i, _)| i)
            .collect()
    }

    fn get_empty_rows(&self) -> Vec<usize> {
        let height = self.height;

        let mut markers = vec![true; height];
        for (h, _) in self.galaxy_index.values() {
            markers[*h] = false;
        }

        markers
            .iter()
            .enumerate()
            .filter(|(_, marked)| **marked)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn parse(input: impl AsRef<Path>) -> AOCResult<SpaceMap> {
        let lines = read_lines_as_bytes(input)?;
        let mut map: Vec<Vec<SpaceArea>> = Vec::new();
        let mut id = 0;

        for line in lines {
            let mut row: Vec<SpaceArea> = Vec::new();
            for cell in line {
                match cell as char {
                    '.' => row.push(SpaceArea::Empty),
                    '#' => {
                        row.push(SpaceArea::Galaxy(id));
                        id += 1;
                    },
                    _ => {
                        return Err(AOCError::ParseError(format!("Invalid space character: {}", cell)));
                    }
                }
            }
            map.push(row);
        }

        Ok(SpaceMap::from(&map))
    }
}

fn run_part(input: impl AsRef<Path>, expansion_amount: usize) -> AOCResult<String> {
    let space_map = SpaceMap::parse(input)?;
    let expanded_space_map = space_map.expand(expansion_amount);

    let g_distances = expanded_space_map.calculate_galaxy_distances();
    let result: usize = g_distances.iter().map(|(_, _, d)| *d).sum();

    Ok(result.to_string())
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    run_part(input, 1)
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    run_part(input, 1_000_000 - 1)
}