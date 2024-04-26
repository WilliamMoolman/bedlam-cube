use crate::puzzle::{Board, Coord, Placement, Puzzle, PuzzlePiece};

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
        puzzle: &Puzzle,
        predicate: Vec<(PuzzlePiece, Board)>,
        board: Board,
        remaining: &Vec<PuzzlePiece>,
    ) {
        if remaining.is_empty() {
            self.add_solution(predicate, true);
            return;
        }

        for (idx, piece) in remaining.iter().enumerate() {
            let mut other_pieces = remaining.clone();
            other_pieces.remove(idx);
            for &placement in piece.placements() {
                if !board.overlaps(placement)
                    && board.overlaps_first_open(placement)
                    && board.union(placement).has_full_coverage(&other_pieces)
                    && board.union(placement).can_pieces_fit(&other_pieces)
                {
                    let mut new_pred = predicate.clone();
                    new_pred.push((piece.clone(), placement));
                    self.solve_board(puzzle, new_pred, board.union(placement), &other_pieces);
                }
            }
        }
    }

    fn solve_corners(
        &mut self,
        puzzle: &Puzzle,
        predicate: Vec<(PuzzlePiece, Board)>,
        board: Board,
        corners: &Vec<Coord>,
        remaining: &Vec<PuzzlePiece>,
    ) {
        let mut new_corners = corners.clone();
        let corner = match new_corners.pop() {
            Some(c) => c,
            None => {
                self.solve_board(puzzle, predicate, board, remaining);
                return;
            }
        };

        for (idx, piece) in remaining.iter().enumerate() {
            let mut leftover = remaining.clone();
            leftover.remove(idx);
            for &placement in piece.placements() {
                let cidx = corner.to_index();
                if placement.get(cidx)
                    && !board.overlaps(placement)
                    && board.union(placement).has_full_coverage(&leftover)
                    && board.union(placement).can_pieces_fit(&leftover)
                {
                    let mut new_pred = predicate.clone();
                    new_pred.push((piece.clone(), placement));
                    self.solve_corners(
                        puzzle,
                        new_pred,
                        board.union(placement),
                        &new_corners,
                        &leftover,
                    );
                }
            }
        }
    }

    pub fn begin(&mut self, puzzle: &Puzzle) {
        self.start_time = Some(Instant::now());
        let corners = puzzle.corners();
        self.solve_corners(puzzle, Vec::new(), Board::new(), &corners, &puzzle.pieces);
    }
}
