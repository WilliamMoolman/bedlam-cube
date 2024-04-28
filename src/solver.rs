use crate::puzzle::{Arrangement, Bitset, Board, Coord, Orientation, Placement, Puzzle};

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

    pub fn has_full_coverage(puzzle: &Puzzle, board: Bitset, pieces: &Vec<usize>) -> bool {
        let mut coverage = board.clone();
        for pid in pieces {
            let piece = &puzzle.pieces[*pid];
            piece
                .placements
                .iter()
                .filter(|placement: &&Placement| !board.intersects(**placement))
                .for_each(|placement: &Placement| coverage = coverage.union(*placement));
        }
        coverage.0 == Board::MAX
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

    pub fn can_pieces_fit(&self, puzzle: &Puzzle, board: Bitset, pieces: &Vec<usize>) -> bool {
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

    // pub fn overlaps_first_open(&self, tmp: Bitset, other: Bitset) -> bool {
    //     let inverted_board = !tmp.0;
    //     let ls0_mask = inverted_board & inverted_board.wrapping_neg();
    //     let open_idx = ls0_mask.trailing_zeros();

    //     (other.0 >> open_idx) & 1 == 1
    // }

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
                    && placement.intersects(mask)
                    && Solver::has_full_coverage(puzzle, new_board, &other_pieces)
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
                    && Solver::has_full_coverage(puzzle, new_board, &leftover)
                    && self.can_pieces_fit(puzzle, new_board, &leftover)
                {
                    arrangement.push(*pid, placement);
                    self.solve_corners(puzzle, arrangement, &new_corners, &leftover);
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
        // println!("{}",Board::from_orientation(&constrained_piece.base));
        let mut unique_rotations: Vec<Board> = Vec::new();
        for placement in constrained_piece.1.placements() {
            //if first { first = false; continue }
            let mut unique = true;
            for orientation in Orientation::from_placement(*placement).get_all_rotations(puzzle.dim)
            {
                // orientation.normalise_to_board(4);
                // println!("{}", Board::from_orientation(&orientation));
                if unique_rotations.contains(&Board::from_orientation(&orientation)) {
                    unique = false;
                    break;
                }
            }
            // return vec![];
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
        let corners = puzzle.corners();
        // let mut arrangement = Arrangement::new();

        let (used_piece, starting_arrangements) = self.constrain_start(puzzle);

        // let remaining = vec![0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let mut remaining: Vec<usize> = (0..puzzle.pieces.len()).collect();
        remaining.remove(used_piece);

        for a in starting_arrangements {
            self.solve_corners(puzzle, &mut a.clone(), &corners, &remaining);
        }
        // arrangement.push(1, Bitset(0x0000000000000272));
        // self.solve_corners(puzzle, &mut arrangement, &corners, &remaining);
        // arrangement.pop();

        // arrangement.push(1, Bitset(0x0000000002720000));
        // self.solve_corners(puzzle, &mut arrangement, &corners, &remaining);
        // arrangement.pop();
    }
}
