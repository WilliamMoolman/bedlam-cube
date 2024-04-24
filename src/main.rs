use colored::*;
use csv;
use itertools::Itertools;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

type Board = u64;

#[derive(Clone)]
struct PuzzlePiece {
    name: String,
    base: Orientation,
    positions: Vec<Board>,
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Coord {
    x: i64,
    y: i64,
    z: i64,
}

impl Coord {
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

#[derive(Clone)]
struct Orientation(Vec<Coord>);

impl Hash for Orientation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Get the bitmask and feed it into the hasher
        self.bitmask().hash(state);
    }
}

impl PartialEq for Orientation {
    fn eq(&self, other: &Self) -> bool {
        // Equality based on the bitmask
        self.bitmask() == other.bitmask()
    }
}

impl Eq for Orientation {}

impl fmt::Debug for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use self.bitmask() to get the bitmask and format it
        write!(f, "{:b}", self.bitmask())
    }
}
impl fmt::Display for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use self.bitmask() to get the bitmask and format it
        write!(f, "{:b}", self.bitmask())
    }
}

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

    fn bitmask(&self) -> Board {
        let mut mask: Board = 0;
        for coord in &self.0 {
            mask |= 1 << ((coord.z as u64) * 16 + (coord.y as u64) * 4 + (coord.x as u64))
        }
        mask
    }
}

impl PuzzlePiece {
    fn new(name: String, base: Orientation) -> PuzzlePiece {
        PuzzlePiece {
            name,
            base,
            positions: vec![],
        }
    }

    fn from_csv(path: PathBuf) -> Result<Vec<PuzzlePiece>, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut pieces = vec![];
        for result in rdr.records() {
            let record = result?;
            let colour = record[1].parse().unwrap_or(Color::BrightRed);
            pieces.push(PuzzlePiece::new(
                record[0].color(colour).to_string(),
                Orientation(Coord::from_str(&record[2])),
            ));
        }
        Ok(pieces)
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
        // println!("{self:?}: ORI {}", unique_orientations.len());
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
                        self.positions.push(new_pos.bitmask());
                    }
                }
            }
        }
        // println!("{self:?}: POS {}", self.positions.len());
    }
}

fn pretty_bitmask(mask: Board) {
    for y in 0..4 {
        for z in 0..4 {
            for x in 0..4 {
                if (((mask >> 16 * z) >> 4 * y) >> x) & 1 == 1 {
                    print!("X");
                } else {
                    print!(".");
                }
            }
            print!(" ");
        }
        println!();
    }
}

fn not_intersect_board(board: Board, piece: Board) -> bool {
    (board & piece) == 0
}

fn entropy(pieces: &Vec<PuzzlePiece>, board: Board) -> f64 {
    // https://en.wikipedia.org/wiki/Entropy_(information_theory)
    let mut sum = 0.0;
    for piece in pieces {
        let length = piece
            .positions
            .iter()
            .filter(|position: &&Board| not_intersect_board(board, **position))
            .count() as f64;
        if length == 0.0 {
            return f64::MIN;
        } else {
            sum += length.log2();
        }
    }
    sum
}

fn check_best_placement(
    pieces: &Vec<PuzzlePiece>,
    board: Board,
) -> Option<((PuzzlePiece, Board), f64)> {
    let mut max_score = f64::MIN;
    let mut best_position = None;
    for (piece_idx, piece) in pieces.iter().enumerate() {
        let mut other_pieces: Vec<PuzzlePiece> = (*pieces.clone()).to_vec();
        other_pieces.remove(piece_idx);
        for position in piece.positions.iter() {
            if not_intersect_board(board, *position) {
                let new_board = board | position;
                let score = entropy(&other_pieces, new_board);
                if score > max_score {
                    max_score = score;
                    best_position = Some((piece.clone(), *position));
                }
            }
        }
    }
    if let Some(best) = best_position {
        Some((best, max_score))
    } else {
        None
    }
}

fn get_best_move(
    pieces: &Vec<PuzzlePiece>,
    board: Board,
    depth: usize,
) -> Option<((PuzzlePiece, Board), f64)> {
    if depth == 1 {
        check_best_placement(pieces, board)
    } else {
        let mut max_score = f64::MIN;
        let mut best_position = None;
        for (piece_idx, piece) in pieces.iter().enumerate() {
            let mut other_pieces: Vec<PuzzlePiece> = (*pieces.clone()).to_vec();
            other_pieces.remove(piece_idx);
            for position in piece.positions.iter() {
                if not_intersect_board(board, *position) {
                    let new_board = board | position;
                    let result = get_best_move(&other_pieces, new_board, depth - 1);
                    if let Some((position_move, score)) = result {
                        if score > max_score {
                            max_score = score;
                            best_position = Some((piece.clone(), *position));
                        }
                    }
                }
            }
        }
        if let Some(best) = best_position {
            Some((best, max_score))
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut pieces = PuzzlePiece::from_csv(PathBuf::from("pieces.csv"))?;

    for piece in pieces.iter_mut() {
        let orientations = piece.generate_unique_orientations();
        piece.compute_possible_positions(&orientations);
    }

    println!("Starting Entropy: {}", entropy(&pieces, 0));

    let mut board = 0;
    let mut remaining_pieces = pieces.clone();
    let mut move_count = 0;
    loop {
        let next_move = get_best_move(&remaining_pieces, board, 2);

        if let Some(((best_piece, pos), score)) = next_move {
            println!("[{move_count}] {best_piece:?} [{pos}]: {score}");
            board |= pos;

            let piece_id = remaining_pieces
                .iter()
                .position(|piece| *piece == best_piece)
                .unwrap();
            remaining_pieces.remove(piece_id);
        } else {
            println!("No possible solutions!");
            println!("Remaining: {remaining_pieces:?}");
            break;
        }
        move_count += 1;
    }

    println!("Board State");
    pretty_bitmask(board);

    Ok(())
}
