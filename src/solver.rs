use crate::puzzle::{Arrangement, Bitset, Board, Coord, Orientation, Placement, Puzzle};

use std::ops::BitAnd;
use std::simd::cmp::SimdPartialEq;
use std::simd::num::SimdUint;
use std::simd::u64x8;
use std::time::Instant;
use rayon::prelude::*;


pub struct Solver {
    start_time: Instant,
}

impl Solver {
    pub fn build() -> Solver {
        Solver {
            start_time: Instant::now(),
        }
    }

    fn process_placement_chunk(board: Board, placements: &u64x8, coverage: u64) -> u64 {
        let intersects = u64x8::splat(board.0).bitand(placements); // SIMD intersection

        let has_intersected = intersects.simd_eq(u64x8::splat(0));

        let selected = has_intersected.select(*placements, u64x8::splat(0));

        let reduced = selected.reduce_or();

        coverage | reduced
    }

    pub fn has_full_coverage(
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

    pub fn can_pieces_fit(puzzle: &Puzzle, board: Bitset, pieces: &Vec<usize>) -> bool {
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
        // &self,
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
        solutions: &mut usize,
        puzzle: &Puzzle,
        arrangement: &mut Arrangement,
        static_arrangement: &Arrangement,
        prev: usize,
        remaining: &Vec<usize>,
    ) {
        if remaining.is_empty() {
            puzzle.show(&[arrangement, static_arrangement]);
            println!();
            *solutions += 1;
            return;
        }

        let (cube, mask) = match Solver::new_cube(puzzle, arrangement, prev) {
            Some((c, m)) => (c, m),
            None => return,
        };

        if remaining.len() == 12 {
            remaining.par_iter().enumerate().map(|(idx, pid)| {
                let mut new_solutions = 0;
                let mut new_arrangement = Arrangement::new();
                new_arrangement.occupied = arrangement.occupied;
                Solver::check_next_piece(&mut new_solutions, puzzle, remaining, idx, *pid, &mut new_arrangement, arrangement, mask, cube); // Check if clone is OK
                new_solutions
            }).collect::<Vec<_>>().iter().for_each(|s| *solutions += s);
        } else {
            for (idx, pid) in remaining.iter().enumerate() {
                Solver::check_next_piece(solutions, puzzle, remaining, idx, *pid, arrangement, static_arrangement, mask, cube);
            }
        }
    }

    fn check_next_piece(solutions: &mut usize, puzzle: &Puzzle, remaining: &Vec<usize>, idx: usize, pid: usize, arrangement: &mut Arrangement, static_arrangement: &Arrangement, mask: Board, cube: usize) {
        let mut other_pieces = remaining.clone();
        other_pieces.remove(idx);
        let piece = &puzzle.pieces[pid];
        for &placement in piece.placements() {
            let new_board = arrangement.occupied.union(placement);
            if !arrangement.occupied.intersects(placement)
                && placement.intersects(mask) // Check if the piece occupies next availiable board position
                && Solver::has_full_coverage(puzzle, new_board, &other_pieces)
                && Solver::can_pieces_fit(puzzle, new_board, &other_pieces)
            {
                arrangement.push(pid, placement);
                Solver::solve_board(solutions, puzzle, arrangement, static_arrangement, cube, &other_pieces);
                arrangement.pop();
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
        self.start_time = Instant::now();

        let (used_piece, starting_arrangements) = self.constrain_start(puzzle);

        let mut remaining: Vec<usize> = (0..puzzle.pieces.len()).collect();
        remaining.remove(used_piece);
        let mut solutions = 0;
        for a in starting_arrangements {
            Solver::solve_board(&mut solutions, puzzle, &mut a.clone(), &Arrangement::new(), 0, &remaining)
        }

        // Print Information
        let duration = Instant::now()
            .duration_since(self.start_time)
            .as_secs();
        let s_per_solution = duration as f64 / solutions as f64;
        println!("\n===== Statistics =====");
        println!("Total Solutions: {}", solutions);
        println!("Total Duration: {}s", duration);
        println!("Rate: {:.2}ms per solution", s_per_solution * 1000.0);
    }
}
