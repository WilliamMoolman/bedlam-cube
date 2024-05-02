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

    #[arg(short, default_value = "4x4x4")]
    size: String,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let puzzle = Puzzle::from_csv(args.puzzle, &args.size)?;
    println!("{:?}", args.size);

    let mut solver = Solver::build();
    solver.begin(&puzzle);
    Ok(())
}
