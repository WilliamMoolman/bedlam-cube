use crate::puzzle::{Board, Coord, Placement, PuzzlePiece};

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
        corners: &Vec<Coord>,
        remaining_pieces: Vec<PuzzlePiece>,
    ) {
        let mut new_corners = corners.clone();
        let corner = match new_corners.pop() {
            Some(c) => c,
            None => {
                self.solve_board(predicate, board, remaining_pieces);
                return;
            }
        };

        for (idx, piece) in remaining_pieces.iter().enumerate() {
            let mut other_pieces = remaining_pieces.clone();
            other_pieces.remove(idx);
            for &placement in piece.placements() {
                if placement.has_coord_set(&corner)
                    && !board.overlaps(placement)
                    && board.union(placement).has_full_coverage(&other_pieces)
                    && board.union(placement).can_pieces_fit(&other_pieces)
                {
                    let mut new_pred = predicate.clone();
                    new_pred.push((piece.clone(), placement));
                    self.solve_corners(
                        new_pred,
                        board.union(placement),
                        &new_corners,
                        other_pieces.clone(),
                    );
                }
            }
        }
    }

    pub fn begin(&mut self, pieces: &Vec<PuzzlePiece>) {
        self.start_time = Some(Instant::now());
        let corners = vec![
            Coord::new(0, 0, 0),
            Coord::new(0, Board::DIMENSION - 1, 0),
            Coord::new(0, 0, Board::DIMENSION - 1),
            Coord::new(0, Board::DIMENSION - 1, Board::DIMENSION - 1),
            Coord::new(Board::DIMENSION - 1, 0, 0),
            Coord::new(Board::DIMENSION - 1, 0, Board::DIMENSION - 1),
            Coord::new(Board::DIMENSION - 1, Board::DIMENSION - 1, 0),
            Coord::new(
                Board::DIMENSION - 1,
                Board::DIMENSION - 1,
                Board::DIMENSION - 1,
            ),
        ];
        self.solve_corners(Vec::new(), Board::new(), &corners, pieces.clone());
    }
}
