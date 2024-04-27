use crate::puzzle::{Arrangement, Bitset, Board, Coord, Placement, Puzzle};

use std::time::Instant;

pub struct Solver {
    total_solutions: usize,
    solutions: Vec<Vec<(usize, Placement)>>,
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

    fn add_solution(&mut self, puzzle: &Puzzle, arrangement: Arrangement, output: bool) {
        let duration = if let Some(start) = self.start_time {
            Instant::now().duration_since(start).as_secs()
        } else {
            0
        };

        puzzle.show(&arrangement);

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

    pub fn has_full_coverage(&self, puzzle: &Puzzle, tmp: Bitset, pieces: &Vec<usize>) -> bool {
        let mut coverage = tmp.clone();
        for pid in pieces {
            let piece = &puzzle.pieces[*pid];
            piece
                .placements
                .iter()
                .filter(|placement: &&Placement| !tmp.intersects(**placement))
                .for_each(|placement: &Placement| coverage = coverage.union(*placement));
        }
        coverage.0 == Board::MAX
    }

    pub fn can_pieces_fit(&self, puzzle: &Puzzle, tmp: Bitset, pieces: &Vec<usize>) -> bool {
        for pid in pieces {
            let piece = &puzzle.pieces[*pid];
            let length = piece
                .placements
                .iter()
                .filter(|placement: &&Placement| !tmp.intersects(**placement))
                .count();
            if length == 0 {
                return false;
            }
        }
        return true;
    }

    // pub fn overlaps_first_open(&self, tmp: Bitset, other: Bitset) -> bool {
    //     let inverted_board = !tmp.0;
    //     let ls0_mask = inverted_board & inverted_board.wrapping_neg();
    //     let open_idx = ls0_mask.trailing_zeros();

    //     (other.0 >> open_idx) & 1 == 1
    // }

    fn new_cube(
        &self,
        puzzle: &Puzzle,
        arrangement: &Arrangement,
        prev: usize,
    ) -> Option<(usize, Bitset)> {
        let mut cube = prev;
        let mut mask = 1 << cube;

        while mask & arrangement.occupied.0 != 0 {
            cube += 1;
            mask <<= 1;
        }

        // do a check to ensure not isolated cube

        Some((cube, Bitset(mask)))
    }

    fn solve_board(
        &mut self,
        puzzle: &Puzzle,
        arrangement: &mut Arrangement,
        prev: usize,
        remaining: &Vec<usize>,
    ) {
        if remaining.is_empty() {
            self.add_solution(puzzle, arrangement.clone(), true);
            return;
        }

        let (cube, mask) = match self.new_cube(puzzle, arrangement, prev) {
            Some((c, m)) => (c, m),
            None => return,
        };

        for (idx, pid) in remaining.iter().enumerate() {
            let mut other_pieces = remaining.clone();
            other_pieces.remove(idx);
            let piece = &puzzle.pieces[*pid];
            for &placement in piece.placements() {
                let new_board = arrangement.occupied.union(placement);
                if !arrangement.occupied.intersects(placement)
                    && placement.intersects(mask)
                    && self.has_full_coverage(puzzle, new_board, &other_pieces)
                    && self.can_pieces_fit(puzzle, new_board, &other_pieces)
                {
                    arrangement.push(*pid, placement);
                    self.solve_board(puzzle, arrangement, cube, &other_pieces);
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
        remaining: &Vec<usize>,
    ) {
        let mut new_corners = corners.clone();
        let corner = match new_corners.pop() {
            Some(c) => c,
            None => {
                self.solve_board(puzzle, arrangement, 0, remaining);
                return;
            }
        };

        for (idx, pid) in remaining.iter().enumerate() {
            let mut leftover = remaining.clone();
            leftover.remove(idx);
            let piece = &puzzle.pieces[*pid];
            for &placement in piece.placements() {
                let cidx = corner.to_index();
                let new_board = arrangement.occupied.union(placement);
                if placement.get(cidx)
                    && !arrangement.occupied.intersects(placement)
                    && self.has_full_coverage(puzzle, new_board, &leftover)
                    && self.can_pieces_fit(puzzle, new_board, &leftover)
                {
                    arrangement.push(*pid, placement);
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

        let remaining = vec![0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        // let remaining: Vec<usize> = (0..puzzle.pieces.len()).collect();

        arrangement.push(1, Bitset(0x0000000000000272));
        self.solve_corners(puzzle, &mut arrangement, &corners, &remaining);
        arrangement.pop();

        arrangement.push(1, Bitset(0x0000000002720000));
        self.solve_corners(puzzle, &mut arrangement, &corners, &remaining);
        arrangement.pop();
    }
}
