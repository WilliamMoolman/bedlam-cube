use std::io;
use std::path::PathBuf;

use bedlam_cube::puzzle::Puzzle;
use bedlam_cube::solver::Solver;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Puzzle file
    puzzle: PathBuf,

    /// Returns solution to sudoku
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let puzzle = Puzzle::from_csv(args.puzzle)?;

    let mut solver = Solver::build();
    solver.begin(&puzzle);
    Ok(())
}
