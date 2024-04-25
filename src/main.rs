use std::error::Error;
use std::path::PathBuf;

use bedlam_cube::puzzle::PuzzlePiece;
use bedlam_cube::solver::Solver;

fn main() -> Result<(), Box<dyn Error>> {
    let pieces = PuzzlePiece::from_csv(PathBuf::from("pieces.csv"))?;

    let mut solver = Solver::build();
    solver.begin(&pieces);
    Ok(())
}
