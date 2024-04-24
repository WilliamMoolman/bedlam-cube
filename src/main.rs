use colored::*;
use csv;
use itertools::Itertools;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::Instant;

type Board = u64;

const DIMENSION: usize = 4;

#[derive(Clone)]
struct PuzzlePiece {
    name: String,
    code: String,
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

    fn set_on_board(&self, board: Board) -> bool {
        (((board >> DIMENSION * DIMENSION * self.z as usize) >> DIMENSION * self.y as usize)
            >> self.x as usize)
            & 1
            == 1
    }

    fn from_corner_idx(corner_idx: usize) -> Coord {
        match corner_idx {
            0 => Coord::new(0, 0, 0),
            1 => Coord::new(0, DIMENSION - 1, 0),
            2 => Coord::new(0, 0, DIMENSION - 1),
            3 => Coord::new(0, DIMENSION - 1, DIMENSION - 1),
            4 => Coord::new(DIMENSION - 1, 0, 0),
            5 => Coord::new(DIMENSION - 1, 0, DIMENSION - 1),
            6 => Coord::new(DIMENSION - 1, DIMENSION - 1, 0),
            7 => Coord::new(DIMENSION - 1, DIMENSION - 1, DIMENSION - 1),
            _ => panic!(),
        }
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
    fn new(name: String, code: String, base: Orientation) -> PuzzlePiece {
        PuzzlePiece {
            name,
            code,
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
                        self.positions.push(new_pos.bitmask());
                    }
                }
            }
        }
    }
}

fn _pretty_bitmask(mask: Board) {
    for y in 0..4 {
        for z in 0..4 {
            for x in 0..4 {
                let c = Coord { x, y, z };
                if c.set_on_board(mask) {
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

fn do_all_pieces_fit(pieces: &Vec<PuzzlePiece>, board: Board) -> bool {
    for piece in pieces {
        let length = piece
            .positions
            .iter()
            .filter(|position: &&Board| not_intersect_board(board, **position))
            .count();
        if length == 0 {
            return false;
        }
    }
    return true;
}

fn can_voxels_be_filled(board: Board, pieces: &Vec<PuzzlePiece>) -> bool {
    let mut coverage = board;
    for piece in pieces {
        piece
            .positions
            .iter()
            .filter(|position: &&Board| not_intersect_board(board.clone(), **position))
            .for_each(|position: &Board| coverage |= position);
    }
    coverage == Board::MAX
}

fn intersects_first_open(position: Board, board: Board) -> bool {
    let inverted_board = !board;
    let ls0_mask = inverted_board & inverted_board.wrapping_neg();
    let open_idx = ls0_mask.trailing_zeros();

    (position >> open_idx) & 1 == 1
}

struct Solver {
    total_solutions: usize,
    solutions: Vec<Vec<(PuzzlePiece, Board)>>,
    start_time: Option<Instant>,
}

impl Solver {
    fn build() -> Solver {
        Solver {
            total_solutions: 0,
            solutions: Vec::new(),
            start_time: None,
        }
    }

    fn add_solution(&mut self, solution: Vec<(PuzzlePiece, Board)>, output: bool) {
        self.total_solutions += 1;
        self.solutions.push(solution.clone());
        if !output {
            return;
        }

        let duration = if let Some(start) = self.start_time {
            Instant::now().duration_since(start).as_secs()
        } else {
            0
        };
        let mut printgrid = [[" "; DIMENSION * (DIMENSION + 1)]; DIMENSION];
        for (piece, position) in &solution {
            for y in 0..4 {
                for z in 0..4 {
                    for x in 0..4 {
                        let c = Coord { x, y, z };
                        if c.set_on_board(*position) {
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

        let s_per_solution = duration as f64 / self.total_solutions as f64;
        println!(
            "Total Solutions: {} [rate {:.2}s per solution]",
            self.total_solutions, s_per_solution
        )
    }

    fn solve_board(
        &mut self,
        predicate: Vec<(PuzzlePiece, Board)>,
        board: Board,
        remaining_pieces: Vec<PuzzlePiece>,
    ) {
        for (idx, piece) in remaining_pieces.iter().enumerate() {
            let mut other_pieces = remaining_pieces.clone();
            other_pieces.remove(idx);
            for position in &piece.positions {
                if not_intersect_board(board, *position)
                    && intersects_first_open(*position, board)
                    && can_voxels_be_filled(board | position, &other_pieces)
                    && do_all_pieces_fit(&other_pieces, board | position)
                {
                    let mut new_pred = predicate.clone();
                    new_pred.push((piece.clone(), *position));
                    if other_pieces.len() == 0 {
                        self.add_solution(new_pred, true);
                    } else {
                        self.solve_board(new_pred, board | position, other_pieces.clone());
                    }
                }
            }
        }
    }

    fn solve_corners(
        &mut self,
        predicate: Vec<(PuzzlePiece, Board)>,
        board: Board,
        corner: usize,
        remaining_pieces: Vec<PuzzlePiece>,
    ) {
        let corner_coord = Coord::from_corner_idx(corner);
        for (idx, piece) in remaining_pieces.iter().enumerate() {
            let mut other_pieces = remaining_pieces.clone();
            other_pieces.remove(idx);
            for position in &piece.positions {
                if corner_coord.set_on_board(*position)
                    && not_intersect_board(board, *position)
                    && can_voxels_be_filled(board | position, &other_pieces)
                    && do_all_pieces_fit(&other_pieces, board | position)
                {
                    let mut new_pred = predicate.clone();
                    new_pred.push((piece.clone(), *position));
                    if new_pred.len() == 8 {
                        self.solve_board(new_pred.clone(), board | position, other_pieces.clone());
                    } else {
                        self.solve_corners(
                            new_pred,
                            board | position,
                            corner + 1,
                            other_pieces.clone(),
                        );
                    }
                }
            }
        }
    }

    fn begin(&mut self, pieces: &Vec<PuzzlePiece>) {
        self.start_time = Some(Instant::now());
        self.solve_corners(Vec::new(), 0, 0, pieces.clone());
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut pieces = PuzzlePiece::from_csv(PathBuf::from("pieces.csv"))?;

    for piece in pieces.iter_mut() {
        let orientations = piece.generate_unique_orientations();
        piece.compute_possible_positions(&orientations);
    }

    let mut solver = Solver::build();
    solver.begin(&pieces);
    Ok(())
}
