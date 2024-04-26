use std::io;
use std::path::PathBuf;

use bedlam_cube::puzzle::{Coord, Puzzle, PuzzlePiece};
use bedlam_cube::solver::Solver;

fn main() -> io::Result<()> {
    let pieces = PuzzlePiece::from_csv(PathBuf::from("pieces.csv"))?;
    let puzzle = Puzzle {
        pieces: pieces,
        dim: Coord::new(4, 4, 4),
    };

    let mut solver = Solver::build();
    solver.begin(&puzzle);
    Ok(())
}
