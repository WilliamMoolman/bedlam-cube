use crate::puzzle::{Board, Coord, Placement, PuzzlePiece, Orientation};

use std::time::Instant;

pub struct Solver {
    total_solutions: usize,
    solutions: Vec<Vec<(PuzzlePiece, Placement)>>,
    start_time: Option<Instant>,
}

impl Solver {
    pub fn build() -> Solver {
        Solver {
            total_solutions: 0,
            solutions: Vec::new(),
            start_time: None,
        }
    }

    fn add_solution(&mut self, solution: Vec<(PuzzlePiece, Placement)>, output: bool) {
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

        Board::print_board(&solution);

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
            for &placement in piece.placements() {
                if !board.overlaps(placement)
                    && board.overlaps_first_open(placement)
                    && board.union(placement).has_full_coverage(&other_pieces)
                    && board.union(placement).can_pieces_fit(&other_pieces)
                {
                    let mut new_pred = predicate.clone();
                    new_pred.push((piece.clone(), placement));
                    if other_pieces.len() == 0 {
                        self.add_solution(new_pred, true);
                    } else {
                        self.solve_board(new_pred, board.union(placement), other_pieces.clone());
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
            for &placement in piece.placements() {
                if placement.has_coord_set(&corner_coord)
                    && !board.overlaps(placement)
                    && board.union(placement).has_full_coverage(&other_pieces)
                    && board.union(placement).can_pieces_fit(&other_pieces)
                {
                    let mut new_pred = predicate.clone();
                    new_pred.push((piece.clone(), placement));
                    if new_pred.len() == 8 {
                        self.solve_board(
                            new_pred.clone(),
                            board.union(placement),
                            other_pieces.clone(),
                        );
                    } else {
                        self.solve_corners(
                            new_pred,
                            board.union(placement),
                            corner + 1,
                            other_pieces.clone(),
                        );
                    }
                }
            }
        }
    }

    fn constrain_start(&self, pieces: &Vec<PuzzlePiece>) -> Vec<Board> {
        let constrained_piece = pieces.iter().min_by(|&p1, &p2| p1.placements().len().cmp(&p2.placements().len())).unwrap(); 
        let mut unique_rotations: Vec<Board> = Vec::new();
        for placement in constrained_piece.placements() {
            let mut unique = true;
            for mut orientation in Orientation::from_placement(*placement).get_all_rotations() {
                orientation.normalise_to_board(4);
                println!("{}", Board::from_orientation(&orientation));
                if unique_rotations.contains(&Board::from_orientation(&orientation)) {
                    unique = false;
                    break;
                }
            }
            return vec![];
            if unique {
                unique_rotations.push(*placement);
            }
        }


        println!("{constrained_piece:?}");
        unique_rotations
    }

    pub fn begin(&mut self, pieces: &Vec<PuzzlePiece>) {
        self.start_time = Some(Instant::now());
        // Constrain rotational symmeteries by finding most constrained piece
        let starting_boards = self.constrain_start(pieces);
        for board in starting_boards {
            println!("{board}");
        }
        // self.solve_corners(Vec::new(), Board::new(), 0, pieces.clone());
    }
}
