use std::collections::HashSet;
use std::path::Path;

use crate::aocio::read_lines_as_bytes;
use crate::aocbase::{AOCResult, AOCError};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    y: i64,
    x: i64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Photon {
    position: Position,
    vector: Position,
}

impl Photon {
    pub fn with_vector(&self, y: i64, x: i64) -> Photon {
        Photon { position: self.position.clone(), vector: Position { y, x } }
    }

    pub fn set_vector(&mut self, y: i64, x: i64) {
        self.vector.y = y;
        self.vector.x = x;
    }

    pub fn move_step(&mut self) {
        self.position.y += self.vector.y;
        self.position.x += self.vector.x;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Reflector {
    Vertical,
    Horizontal,
    DiagonallLeft,
    DiagonalRight,
    Space,
}

impl Reflector {

    pub fn apply(&self, mut photon: Photon) -> Vec<Photon> {
        match self {
            Reflector::Vertical => {
                if photon.vector.x != 0 {
                    photon.set_vector(1, 0);
                    vec![photon, photon.with_vector(-1, 0)]
                }
                else {
                    vec![photon]
                }
            },
            Reflector::Horizontal => {
                if photon.vector.y != 0 {
                    photon.set_vector(0, 1);
                    vec![photon, photon.with_vector(0, -1)]
                }
                else {
                    vec![photon]
                }
            },
            Reflector::DiagonallLeft => {
                photon.set_vector(photon.vector.x, photon.vector.y);
                vec![photon]
            },
            Reflector::DiagonalRight => {
                photon.set_vector(photon.vector.x * -1, photon.vector.y * -1);
                vec![photon]
            },
            Reflector::Space => {
                vec![photon]
            }
        }
    }

    pub fn parse(c: char) -> AOCResult<Reflector> {
        Ok(match c {
            '|' => Reflector::Vertical,
            '-' => Reflector::Horizontal,
            '\\' => Reflector::DiagonallLeft,
            '/' => Reflector::DiagonalRight,
            '.' => Reflector::Space,
            _ => { return Err(AOCError::ParseError(format!("Invalid character: {}", c))); }
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Tile {
    pub reflector: Reflector,
    pub energized: i32,
}

impl Tile {
    pub fn new(reflector: Reflector) -> Self {
        Tile { reflector, energized: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct ReflectionGrid {
    pub tiles: Vec<Vec<Tile>>,
}

impl ReflectionGrid {

    pub fn new(tiles: Vec<Vec<Tile>>) -> ReflectionGrid {
        ReflectionGrid { tiles }
    }

    pub fn height(&self) -> i64 {
        self.tiles.len() as i64
    }

    pub fn width(&self) -> i64 {
        self.tiles[0].len() as i64
    }

    pub fn parse(input: impl AsRef<Path>) -> AOCResult<ReflectionGrid> {
        let lines = read_lines_as_bytes(input)?;
        let mut tiles: Vec<Vec<Tile>> = Vec::new();

        for row in lines {
            tiles.push(row
                .iter()
                .map(|c| {
                    Ok(Tile::new(Reflector::parse(*c as char)?))
                })
                .collect::<AOCResult<Vec<Tile>>>()?
            );
        }

        Ok(ReflectionGrid::new(tiles))
    }

    pub fn send_photon(&mut self, photon: &Photon) {
        let mut visitor = PhotonVisitor::new(self);
        visitor.visit(photon);
    }

    pub fn get_energized_count(&self) -> i64 {
        let mut count: i64 = 0;
        for row in &self.tiles {
            for tile in row {
                if tile.energized > 0 {
                    count += 1;
                }
            }
        }
        count
    }
}

pub struct PhotonVisitor<'a> {
    pub reflection_grid: &'a mut ReflectionGrid,
    visited: HashSet<Photon>,
    photons: Vec<Photon>,
}

impl<'a> PhotonVisitor<'a> {
    pub fn new(reflection_grid: &'a mut ReflectionGrid) -> Self {
        Self {
            reflection_grid,
            visited: HashSet::new(),
            photons: Vec::new(),
        }
    }

    pub fn visit(&mut self, photon: &Photon) {
        if !self.visited.contains(photon) {
            self.photons.push(photon.clone());
        }

        let height = self.reflection_grid.height();
        let width = self.reflection_grid.width();

        while let Some(photon) = self.photons.pop() {

            let tile = &mut (self.reflection_grid
                .tiles
                [photon.position.y as usize]
                [photon.position.x as usize]);

            tile.energized += 1;

            let mut photons = tile.reflector.apply(photon);
            for photon in photons.iter_mut() {
                photon.move_step();

                if photon.position.x >= 0 && photon.position.x < width &&
                    photon.position.y >= 0 && photon.position.y < height &&
                    !self.visited.contains(photon)
                {
                    self.visited.insert(photon.clone());
                    self.photons.push(*photon);
                }
            }
        }
    }
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let mut reflection_grid = ReflectionGrid::parse(input)?;

    let initial_photon = Photon {
        position: Position { y: 0, x: 0 },
        vector: Position { y: 0, x: 1 },
    };

    reflection_grid.send_photon(&initial_photon);

    let result = reflection_grid.get_energized_count();

    Ok(result.to_string())
}