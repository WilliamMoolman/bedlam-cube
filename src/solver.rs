use crate::puzzle::{Bitset, Board, Coord, Placement, Puzzle, PuzzlePiece};

use std::time::Instant;

pub struct Solver {
    total_solutions: usize,
    solutions: Vec<Vec<(PuzzlePiece, Placement)>>,
    start_time: Option<Instant>,
}

#[derive(Clone)]
pub struct Arrangement {
    pub occupied: Bitset,
    pub placements: Vec<(PuzzlePiece, Bitset)>,
}

impl Arrangement {
    pub fn new() -> Arrangement {
        Arrangement {
            occupied: Bitset::new(),
            placements: vec![],
        }
    }

    pub fn push(&mut self, piece: PuzzlePiece, placement: Bitset) {
        self.occupied = self.occupied.union(placement);
        self.placements.push((piece, placement));
    }

    pub fn pop(&mut self) -> Option<(PuzzlePiece, Bitset)> {
        match self.placements.pop() {
            Some((piece, placement)) => {
                self.occupied = self.occupied.xor(placement);
                Some((piece, placement))
            }
            None => None,
        }
    }
}

impl Solver {
    pub fn build() -> Solver {
        Solver {
            total_solutions: 0,
            solutions: Vec::new(),
            start_time: None,
        }
    }

    fn add_solution(&mut self, arrangement: Arrangement, output: bool) {
        let duration = if let Some(start) = self.start_time {
            Instant::now().duration_since(start).as_secs()
        } else {
            0
        };

        Board::print_board(&arrangement.placements);

        self.total_solutions += 1;
        self.solutions.push(arrangement.placements);
        if !output {
            return;
        }

        let s_per_solution = duration as f64 / self.total_solutions as f64;
        println!(
            "Total Solutions: {} [rate {:.2}s per solution]",
            self.total_solutions, s_per_solution
        )
    }

    fn solve_board(
        &mut self,
        puzzle: &Puzzle,
        arrangement: &mut Arrangement,
        remaining: &Vec<PuzzlePiece>,
    ) {
        if remaining.is_empty() {
            self.add_solution(arrangement.clone(), true);
            return;
        }

        for (idx, piece) in remaining.iter().enumerate() {
            let mut other_pieces = remaining.clone();
            other_pieces.remove(idx);
            for &placement in piece.placements() {
                if !arrangement.occupied.intersects(placement)
                    && arrangement.occupied.overlaps_first_open(placement)
                    && arrangement
                        .occupied
                        .union(placement)
                        .has_full_coverage(&other_pieces)
                    && arrangement
                        .occupied
                        .union(placement)
                        .can_pieces_fit(&other_pieces)
                {
                    arrangement.push(piece.clone(), placement);
                    self.solve_board(puzzle, arrangement, &other_pieces);
                    arrangement.pop();
                }
            }
        }
    }

    fn solve_corners(
        &mut self,
        puzzle: &Puzzle,
        arrangement: &mut Arrangement,
        corners: &Vec<Coord>,
        remaining: &Vec<PuzzlePiece>,
    ) {
        let mut new_corners = corners.clone();
        let corner = match new_corners.pop() {
            Some(c) => c,
            None => {
                self.solve_board(puzzle, arrangement, remaining);
                return;
            }
        };

        for (idx, piece) in remaining.iter().enumerate() {
            let mut leftover = remaining.clone();
            leftover.remove(idx);
            for &placement in piece.placements() {
                let cidx = corner.to_index();
                if placement.get(cidx)
                    && !arrangement.occupied.intersects(placement)
                    && arrangement
                        .occupied
                        .union(placement)
                        .has_full_coverage(&leftover)
                    && arrangement
                        .occupied
                        .union(placement)
                        .can_pieces_fit(&leftover)
                {
                    arrangement.push(piece.clone(), placement);
                    self.solve_corners(puzzle, arrangement, &new_corners, &leftover);
                    arrangement.pop();
                }
            }
        }
    }

    pub fn begin(&mut self, puzzle: &Puzzle) {
        self.start_time = Some(Instant::now());
        let corners = puzzle.corners();
        let mut arrangement = Arrangement::new();

        self.solve_corners(puzzle, &mut arrangement, &corners, &puzzle.pieces);
    }
}
