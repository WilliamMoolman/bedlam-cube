use colored::*;
use itertools::Itertools;
use std::fmt::format;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::{fmt, io};

#[derive(Clone, Copy)]
pub struct Bitset(pub u64);

pub type Board = Bitset;
pub type Placement = Bitset;

impl fmt::Display for Bitset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use self.bitmask() to get the bitmask and format it
        for y in 0..4 {
            for z in 0..4 {
                for x in 0..4 {
                    let c = Coord { x, y, z }.to_index();
                    if self.get(c) {
                        write!(f, "X")?;
                    } else {
                        write!(f, ".")?;
                    }
                }
                write!(f, " ")?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}

impl Bitset {
    pub const MAX: u64 = u64::MAX;
    pub const DIMENSION: usize = 4;

    pub fn new() -> Bitset {
        Bitset(0)
    }

    fn from_orientation(orientation: &Orientation) -> Bitset {
        let mut mask = Bitset(0);
        for coord in &orientation.0 {
            mask.0 |= 1 << ((coord.z as u64) * 16 + (coord.y as u64) * 4 + (coord.x as u64))
        }
        mask
    }

    pub fn get(&self, index: usize) -> bool {
        (self.0 >> index) & 1 == 1
    }

    pub fn set(&mut self, index: usize) {
        self.0 |= 1 << index;
    }

    pub fn intersects(&self, other: Bitset) -> bool {
        (self.0 & other.0) != 0
    }

    pub fn xor(&self, other: Bitset) -> Bitset {
        Bitset(self.0 ^ other.0)
    }

    pub fn union(&self, other: Bitset) -> Bitset {
        Bitset(self.0 | other.0)
    }

    pub fn intersection(&self, other: Bitset) -> Bitset {
        Bitset(self.0 & other.0)
    }
}

#[derive(Clone)]
pub struct Piece {
    pub name: String,
    pub code: String,
    pub base: Orientation,
    pub placements: Vec<Placement>,
}

impl PartialEq for Piece {
    fn eq(&self, other: &Self) -> bool {
        // Equality based on the bitmask
        self.name == other.name
    }
}

impl Eq for Piece {}

impl fmt::Debug for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("Piece {{ {: ^20} }}", self.name))
    }
}
impl Piece {
    fn new(name: String, code: String, base: Orientation) -> Piece {
        let mut piece = Piece {
            name,
            code,
            base,
            placements: vec![],
        };
        let orientations = piece.generate_unique_orientations();
        piece.compute_possible_positions(&orientations);
        piece
    }

    pub fn placements(&self) -> &Vec<Placement> {
        &self.placements
    }

    fn generate_unique_orientations(&mut self) -> Vec<Orientation> {
        // Has six faces
        // Each face can be in four rotations
        // Good resource: https://www.euclideanspace.com/maths/geometry/rotations/euler/examples/index.htm
        //      Matrix rep: https://www.euclideanspace.com/maths/algebra/matrix/transforms/examples/index.htm
        let mut current_orientation = self.base.clone();
        let mut orientations: Vec<Orientation> = vec![];
        for _ in 0..4 {
            orientations.push(current_orientation.clone());
            let mut o = current_orientation.clone();
            o.rotate(0, 1, 0);
            orientations.push(o);
            let mut o = current_orientation.clone();
            o.rotate(0, 3, 0);
            orientations.push(o);
            let mut o = current_orientation.clone();
            o.rotate(0, 0, 1);
            orientations.push(o);
            let mut o = current_orientation.clone();
            o.rotate(0, 0, 2);
            orientations.push(o);
            let mut o = current_orientation.clone();
            o.rotate(0, 0, 3);
            orientations.push(o);

            current_orientation.rotate(1, 0, 0);
        }
        let unique_orientations: Vec<Orientation> =
            orientations.iter().unique().map(|x| x.clone()).collect();
        unique_orientations
    }

    fn compute_possible_positions(&mut self, unique_orientations: &Vec<Orientation>) {
        for orientation in unique_orientations {
            let x_bound = orientation.0.iter().map(|coord| coord.x).max().unwrap();
            let y_bound = orientation.0.iter().map(|coord| coord.y).max().unwrap();
            let z_bound = orientation.0.iter().map(|coord| coord.z).max().unwrap();
            for x_off in 0..(4 - x_bound) {
                for y_off in 0..(4 - y_bound) {
                    for z_off in 0..(4 - z_bound) {
                        let mut new_pos = orientation.clone();
                        new_pos.0.iter_mut().for_each(|coord| {
                            coord.x += x_off;
                            coord.y += y_off;
                            coord.z += z_off;
                        });
                        self.placements.push(Placement::from_orientation(&new_pos));
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
struct Orientation(Vec<Coord>);

impl Hash for Orientation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Get the bitmask and feed it into the hasher
        let placement = Placement::from_orientation(self);
        placement.0.hash(state);
    }
}

impl PartialEq for Orientation {
    fn eq(&self, other: &Self) -> bool {
        // Equality based on the bitmask
        let placement_a = Placement::from_orientation(self);
        let placement_b = Placement::from_orientation(other);
        placement_a.0 == placement_b.0
    }
}

impl Eq for Orientation {}

// impl fmt::Debug for Orientation {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         // Use self.bitmask() to get the bitmask and format it
//         write!(f, "{:b}", self.bitmask())
//     }
// }
// impl fmt::Display for Orientation {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         // Use self.bitmask() to get the bitmask and format it
//         write!(f, "{:b}", self.bitmask())
//     }
// }

impl Orientation {
    fn rotate(&mut self, x: usize, y: usize, z: usize) {
        // Rotate
        for _ in 0..x {
            self.0.iter_mut().for_each(|coord| coord.rotate_x());
        }
        for _ in 0..y {
            self.0.iter_mut().for_each(|coord| coord.rotate_y());
        }
        for _ in 0..z {
            self.0.iter_mut().for_each(|coord| coord.rotate_z());
        }

        // Normalise
        let min_x = self.0.iter().map(|coord| coord.x).min().unwrap();
        let min_y = self.0.iter().map(|coord| coord.y).min().unwrap();
        let min_z = self.0.iter().map(|coord| coord.z).min().unwrap();

        self.0
            .iter_mut()
            .for_each(|coord| coord.x = coord.x - min_x);
        self.0
            .iter_mut()
            .for_each(|coord| coord.y = coord.y - min_y);
        self.0
            .iter_mut()
            .for_each(|coord| coord.z = coord.z - min_z);
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Coord {
    x: i64,
    y: i64,
    z: i64,
}

impl Coord {
    pub fn new(x: usize, y: usize, z: usize) -> Coord {
        Coord {
            x: x as i64,
            y: y as i64,
            z: z as i64,
        }
    }

    pub fn to_index(&self) -> usize {
        (self.z * 16 + self.y * 4 + self.x) as usize
    }

    fn from_str(s: &str) -> Vec<Coord> {
        s.split("-")
            .map(|coord_s| {
                let mut coord_ids = coord_s.chars();
                Coord {
                    x: coord_ids.next().unwrap().to_digit(10).unwrap() as i64,
                    y: coord_ids.next().unwrap().to_digit(10).unwrap() as i64,
                    z: coord_ids.next().unwrap().to_digit(10).unwrap() as i64,
                }
            })
            .collect()
    }

    fn rotate_x(&mut self) {
        // [ 1  0  0
        //   0  0 -1
        //   0  1  0 ]
        let new_x = self.x;
        let new_y = -self.z;
        let new_z = self.y;
        self.x = new_x;
        self.y = new_y;
        self.z = new_z;
    }
    fn rotate_y(&mut self) {
        // [ 0  0  1
        //   0  1  0
        //  -1  0  0 ]
        let new_x = self.z;
        let new_y = self.y;
        let new_z = -self.x;
        self.x = new_x;
        self.y = new_y;
        self.z = new_z;
    }
    fn rotate_z(&mut self) {
        // [ 0 -1  0
        //   1  0  0
        //   0  0  1 ]
        let new_x = -self.y;
        let new_y = self.x;
        let new_z = self.z;
        self.x = new_x;
        self.y = new_y;
        self.z = new_z;
    }
}

pub struct Puzzle {
    pub name: String,
    pub pieces: Vec<Piece>,
    pub dim: Coord,
}

impl Puzzle {
    pub fn from_csv(path: PathBuf) -> io::Result<Self> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut pieces = vec![];
        for (idx, result) in rdr.records().enumerate() {
            let record = result?;
            let color = record[1].parse().unwrap_or(Color::BrightRed);
            pieces.push(Piece::new(
                record[0].color(color).to_string(),
                format!("{:x}", idx).color(color).to_string(),
                Orientation(Coord::from_str(&record[2])),
            ));
        }
        Ok(Puzzle {
            name: "Bedlam Cube".to_string(),
            pieces,
            dim: Coord::new(4, 4, 4),
        })
    }

    pub fn corners(&self) -> Vec<Coord> {
        vec![
            Coord::new(0, 0, 0),
            Coord::new(Board::DIMENSION - 1, 0, 0),
            Coord::new(0, Board::DIMENSION - 1, 0),
            Coord::new(Board::DIMENSION - 1, Board::DIMENSION - 1, 0),
            Coord::new(0, 0, Board::DIMENSION - 1),
            Coord::new(Board::DIMENSION - 1, 0, Board::DIMENSION - 1),
            Coord::new(0, Board::DIMENSION - 1, Board::DIMENSION - 1),
            Coord::new(
                Board::DIMENSION - 1,
                Board::DIMENSION - 1,
                Board::DIMENSION - 1,
            ),
        ]
        .iter()
        .rev()
        .cloned()
        .collect()
    }

    pub fn show(&self, arrangement: &Arrangement) {
        for y in (0..self.dim.y).rev() {
            for z in 0..self.dim.z {
                for x in 0..self.dim.x {
                    let index = z * self.dim.y * self.dim.x + y * self.dim.x + x;
                    if arrangement.occupied.get(index as usize) {
                        for (id, bits) in arrangement.placements.iter() {
                            if bits.get(index as usize) {
                                // print!("{} ", self.pieces[*id].colored_id());
                                print!("{} ", self.pieces[*id].code);
                                break;
                            }
                        }
                    } else {
                        print!(". ");
                    }
                }
                print!("  ");
            }
            println!();
        }
    }
}

#[derive(Clone)]
pub struct Arrangement {
    pub occupied: Bitset,
    pub placements: Vec<(usize, Bitset)>,
}

impl Arrangement {
    pub fn new() -> Arrangement {
        Arrangement {
            occupied: Bitset::new(),
            placements: vec![],
        }
    }

    pub fn push(&mut self, piece: usize, placement: Bitset) {
        self.occupied = self.occupied.union(placement);
        self.placements.push((piece, placement));
    }

    pub fn pop(&mut self) -> Option<(usize, Bitset)> {
        match self.placements.pop() {
            Some((piece, placement)) => {
                self.occupied = self.occupied.xor(placement);
                Some((piece, placement))
            }
            None => None,
        }
    }
}
