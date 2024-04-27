use colored::*;
use itertools::Itertools;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq)]
pub struct Bitset(u64);

pub type Board = Bitset;
pub type Placement = Bitset;

impl fmt::Display for Bitset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use self.bitmask() to get the bitmask and format it
        for y in 0..4 {
            for z in 0..4 {
                for x in 0..4 {
                    let c = Coord { x, y, z };
                    if self.has_coord_set(&c) {
                        if let fmt::Result::Err(e) = write!(f, "X") {
                            return fmt::Result::Err(e);
                        }
                    } else {
                        if let fmt::Result::Err(e) = write!(f, ".") {
                            return fmt::Result::Err(e);
                        }
                    }
                }
                if let fmt::Result::Err(e) = write!(f, " ") {
                    return fmt::Result::Err(e);
                }
            }
            if let fmt::Result::Err(e) = write!(f, "\n") {
                return fmt::Result::Err(e);
            }
        }
        Ok(())
    }
}

impl Bitset {
    const MAX: u64 = u64::MAX;
    const DIMENSION: usize = 4;

    pub fn new() -> Bitset {
        Bitset(0)
    }

    pub fn from_orientation(orientation: &Orientation) -> Bitset {
        let mut mask = Bitset(0);
        for coord in &orientation.0 {
            mask.0 |= 1 << ((coord.z as u64) * 16 + (coord.y as u64) * 4 + (coord.x as u64))
        }
        mask
    }
    pub fn overlaps(&self, other: Bitset) -> bool {
        (self.0 & other.0) != 0
    }

    pub fn print_board(moves: &Vec<(PuzzlePiece, Placement)>) {
        let mut printgrid = [[" "; Board::DIMENSION * (Board::DIMENSION + 1)]; Board::DIMENSION];
        for (piece, placement) in moves {
            for y in 0..4 {
                for z in 0..4 {
                    for x in 0..4 {
                        let c = Coord { x, y, z };
                        if placement.has_coord_set(&c) {
                            printgrid[y as usize][(z * 5 + x) as usize] = &piece.code;
                        }
                    }
                }
            }
        }
        for row in printgrid {
            for c in row {
                print!("{c}");
            }
            println!();
        }
    }

    pub fn can_pieces_fit(&self, pieces: &Vec<PuzzlePiece>) -> bool {
        for piece in pieces {
            let length = piece
                .placements
                .iter()
                .filter(|placement: &&Placement| !self.overlaps(**placement))
                .count();
            if length == 0 {
                return false;
            }
        }
        return true;
    }

    pub fn union(&self, other: Bitset) -> Bitset {
        Bitset(self.0 | other.0)
    }

    pub fn has_full_coverage(&self, pieces: &Vec<PuzzlePiece>) -> bool {
        let mut coverage = self.clone();
        for piece in pieces {
            piece
                .placements
                .iter()
                .filter(|placement: &&Placement| !self.overlaps(**placement))
                .for_each(|placement: &Placement| coverage = coverage.union(*placement));
        }
        coverage.0 == Board::MAX
    }

    pub fn overlaps_first_open(&self, other: Bitset) -> bool {
        let inverted_board = !self.0;
        let ls0_mask = inverted_board & inverted_board.wrapping_neg();
        let open_idx = ls0_mask.trailing_zeros();

        (other.0 >> open_idx) & 1 == 1
    }

    pub fn has_coord_set(&self, coord: &Coord) -> bool {
        (((self.0 >> Self::DIMENSION * Self::DIMENSION * coord.z as usize)
            >> Self::DIMENSION * coord.y as usize)
            >> coord.x as usize)
            & 1
            == 1
    }
}

#[derive(Clone)]
pub struct PuzzlePiece {
    name: String,
    code: String,
    base: Orientation,
    placements: Vec<Placement>,
}

impl PartialEq for PuzzlePiece {
    fn eq(&self, other: &Self) -> bool {
        // Equality based on the bitmask
        self.name == other.name
    }
}

impl Eq for PuzzlePiece {}

impl fmt::Debug for PuzzlePiece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("PuzzlePiece {{ {: ^20} }}", self.name))
    }
}
impl PuzzlePiece {
    fn new(name: String, code: String, base: Orientation) -> PuzzlePiece {
        let mut piece = PuzzlePiece {
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

    pub fn from_csv(path: PathBuf) -> Result<Vec<PuzzlePiece>, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut pieces = vec![];
        for result in rdr.records() {
            let record = result?;
            let colour = record[2].parse().unwrap_or(Color::BrightRed);
            pieces.push(PuzzlePiece::new(
                record[0].color(colour).to_string(),
                record[1].color(colour).to_string(),
                Orientation(Coord::from_str(&record[3])),
            ));
        }
        Ok(pieces)
    }

    fn generate_unique_orientations(&mut self) -> Vec<Orientation> {
        // Has six faces
        // Each face can be in four rotations
        // Good resource: https://www.euclideanspace.com/maths/geometry/rotations/euler/examples/index.htm
        //      Matrix rep: https://www.euclideanspace.com/maths/algebra/matrix/transforms/examples/index.htm
        let mut orientations = self.base.get_all_rotations();
        orientations.iter_mut().for_each(|o| o.normalise());

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
pub struct Orientation(Vec<Coord>);

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
    pub fn from_placement(placement: Placement) -> Orientation {
        let mut coords = Vec::new();
        for y in 0..4 {
            for z in 0..4 {
                for x in 0..4 {
                    let c = Coord { x, y, z };
                    if placement.has_coord_set(&c) {
                        coords.push(c);
                    }
                }
            }
        }
        Orientation(coords)
    }

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

    }

    pub fn get_all_rotations(&self) -> Vec<Orientation> {
        let mut current_orientation = self.clone();
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

        orientations
    }

    fn normalise(&mut self) {
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
    
    pub fn normalise_to_board(&mut self, dimension: i64) {
        self.0
            .iter_mut()
            .for_each(|coord| coord.x = coord.x.rem_euclid(dimension));
        self.0
            .iter_mut()
            .for_each(|coord| coord.y = coord.y.rem_euclid(dimension));
        self.0
            .iter_mut()
            .for_each(|coord| coord.z = coord.z.rem_euclid(dimension));
    }
        
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Coord {
    x: i64,
    y: i64,
    z: i64,
}

impl Coord {
    fn new(x: usize, y: usize, z: usize) -> Coord {
        Coord {
            x: x as i64,
            y: y as i64,
            z: z as i64,
        }
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

    pub fn from_corner_idx(corner_idx: usize) -> Coord {
        match corner_idx {
            0 => Coord::new(0, 0, 0),
            1 => Coord::new(0, Board::DIMENSION - 1, 0),
            2 => Coord::new(0, 0, Board::DIMENSION - 1),
            3 => Coord::new(0, Board::DIMENSION - 1, Board::DIMENSION - 1),
            4 => Coord::new(Board::DIMENSION - 1, 0, 0),
            5 => Coord::new(Board::DIMENSION - 1, 0, Board::DIMENSION - 1),
            6 => Coord::new(Board::DIMENSION - 1, Board::DIMENSION - 1, 0),
            7 => Coord::new(
                Board::DIMENSION - 1,
                Board::DIMENSION - 1,
                Board::DIMENSION - 1,
            ),
            _ => panic!(),
        }
    }
}
