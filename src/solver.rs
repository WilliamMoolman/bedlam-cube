use crate::puzzle::{Arrangement, Bitset, Board, Coord, Orientation, Placement, Puzzle};

use std::ops::BitAnd;
use std::simd::cmp::SimdPartialEq;
use std::simd::num::SimdUint;
use std::simd::u64x8;
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
            "Total Solutions: {} [rate {:.2}ms per solution]",
            self.total_solutions,
            s_per_solution * 1000.0
        )
    }

    fn process_placement_chunk(board: Board, placements: &u64x8, coverage: u64) -> u64 {
        let intersects = u64x8::splat(board.0).bitand(placements); // SIMD intersection

        let has_intersected = intersects.simd_eq(u64x8::splat(0));

        let selected = has_intersected.select(*placements, u64x8::splat(0));

        let reduced = selected.reduce_or();

        coverage | reduced
    }

    pub fn has_full_coverage(
        &mut self,
        puzzle: &Puzzle,
        board: Bitset,
        pieces: &Vec<usize>,
    ) -> bool {
        let mut coverage = board.clone().0;

        for pid in pieces {
            let piece = &puzzle.pieces[*pid];
            for chunk in piece.simd_placements() {
                coverage = Self::process_placement_chunk(board, chunk, coverage);

                if coverage == Board::MAX {
                    return true;
                }
            }
        }

        coverage == Board::MAX
    }

    pub fn number_orientations_for_coord(
        puzzle: &Puzzle,
        board: Bitset,
        pieces: &Vec<usize>,
        coord: Coord,
    ) -> usize {
        pieces
            .iter()
            .map(|pid| {
                let piece = &puzzle.pieces[*pid];
                piece
                    .placements
                    .iter()
                    .filter(|placement: &&Placement| !board.intersects(**placement))
                    .filter(|placement: &&Placement| placement.get(coord.to_index()))
                    .count()
            })
            .sum()
    }

    pub fn can_pieces_fit(&mut self, puzzle: &Puzzle, board: Bitset, pieces: &Vec<usize>) -> bool {
        for pid in pieces {
            let piece = &puzzle.pieces[*pid];
            let length = piece
                .placements
                .iter()
                .filter(|placement: &&Placement| !board.intersects(**placement))
                .count();
            if length == 0 {
                return false;
            }
        }
        return true;
    }

    fn new_cube(
        &self,
        _puzzle: &Puzzle,
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
                    && placement.intersects(mask) // Check if the piece occupies next availiable board position
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

    fn constrain_start(&self, puzzle: &Puzzle) -> (usize, Vec<Arrangement>) {
        let constrained_piece = puzzle
            .pieces
            .iter()
            .enumerate()
            .min_by(|(_, ref p1), (_, ref p2)| p1.placements().len().cmp(&p2.placements().len()))
            .unwrap();
        let mut unique_rotations: Vec<Board> = Vec::new();
        for placement in constrained_piece.1.placements() {
            let mut unique = true;
            for orientation in Orientation::from_placement(*placement).get_all_rotations(puzzle.dim)
            {
                if unique_rotations.contains(&Board::from_orientation(&orientation)) {
                    unique = false;
                    break;
                }
            }
            if unique {
                unique_rotations.push(*placement);
            }
        }

        let mut starting_arrangements = Vec::new();

        for placement in unique_rotations {
            let mut min_placements_count = usize::MAX;
            let mut min_placements = Placement::new();
            for rotation in Orientation::from_placement(placement).get_all_rotations(puzzle.dim) {
                let board = Board::from_orientation(&rotation);
                let mut pieces: Vec<usize> = (0..puzzle.pieces.len()).collect();
                pieces.remove(constrained_piece.0);
                let placement_count = Solver::number_orientations_for_coord(
                    puzzle,
                    board,
                    &pieces,
                    Coord::new(0, 0, 0),
                );
                if placement_count < min_placements_count {
                    min_placements = board;
                    min_placements_count = placement_count;
                }
            }
            let mut a = Arrangement::new();
            a.push(constrained_piece.0, min_placements);
            starting_arrangements.push(a)
        }

        println!("{constrained_piece:?}");
        (constrained_piece.0, starting_arrangements)
    }

    pub fn begin(&mut self, puzzle: &Puzzle) {
        self.start_time = Some(Instant::now());

        let (used_piece, starting_arrangements) = self.constrain_start(puzzle);

        let mut remaining: Vec<usize> = (0..puzzle.pieces.len()).collect();
        remaining.remove(used_piece);

        for a in starting_arrangements {
            self.solve_board(puzzle, &mut a.clone(), 0, &remaining)
        }

        // Print Information
        let duration = Instant::now()
            .duration_since(self.start_time.unwrap())
            .as_secs();
        let s_per_solution = duration as f64 / self.total_solutions as f64;
        println!("\n===== Statistics =====");
        println!("Total Solutions: {}", self.total_solutions);
        println!("Total Duration: {}s", duration);
        println!("Rate: {:.2}ms per solution", s_per_solution * 1000.0);
    }
}
