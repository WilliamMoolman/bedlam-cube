use colored::*;
use csv;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::path::PathBuf;
use itertools::Itertools;
use std::hash::{Hash, Hasher};

struct PuzzlePiece {
    name: String,
    base: Orientation,
    orientations: Vec<Orientation>,
    permutations: Vec<Orientation>
}

impl fmt::Debug for PuzzlePiece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("PuzzlePiece {{ {: ^20} }}", self.name))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Coord {
    x: i64,
    y: i64,
    z: i64,
}

impl Coord {
    fn from_str(s: &str) -> Vec<Coord> {
        s.split("-")
            .map(|coord_s| {
                let mut coord_ids = coord_s.chars();
                Coord {
                    x: coord_ids.next().unwrap().to_digit(10).unwrap() as i64,
                    y: coord_ids.next().unwrap().to_digit(10).unwrap() as i64,
                    z: coord_ids.next().unwrap().to_digit(10).unwrap() as i64,
                }
            })
            .collect()
    }

    fn rotate_x(&mut self) {
        // [ 1  0  0
        //   0  0 -1
        //   0  1  0 ]
        let new_x = self.x;
        let new_y = -self.z;
        let new_z = self.y;
        self.x = new_x;
        self.y = new_y;
        self.z = new_z;
    }
    fn rotate_y(&mut self) {
        // [ 0  0  1
        //   0  1  0
        //  -1  0  0 ]
        let new_x = self.z;
        let new_y = self.y;
        let new_z = -self.x;
        self.x = new_x;
        self.y = new_y;
        self.z = new_z;
    }
    fn rotate_z(&mut self) {
        // [ 0 -1  0
        //   1  0  0
        //   0  0  1 ]
        let new_x = -self.y;
        let new_y = self.x;
        let new_z = self.z;
        self.x = new_x;
        self.y = new_y;
        self.z = new_z;
    }
}

#[derive(Clone)]
struct Orientation(Vec<Coord>);

impl Hash for Orientation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Get the bitmask and feed it into the hasher
        self.bitmask().hash(state);
    }
}

impl PartialEq for Orientation {
    fn eq(&self, other: &Self) -> bool {
        // Equality based on the bitmask
        self.bitmask() == other.bitmask()
    }
}

impl Eq for Orientation {}

impl fmt::Debug for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use self.bitmask() to get the bitmask and format it
        write!(f, "{:b}", self.bitmask())
    }
}
impl fmt::Display for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use self.bitmask() to get the bitmask and format it
        write!(f, "{:b}", self.bitmask())
    }
}

impl Orientation {
    fn rotate(&mut self, x: usize, y: usize, z: usize) {
        // Rotate
        for _ in 0..x {
            self.0.iter_mut().for_each(|coord | coord.rotate_x());
        }
        for _ in 0..y {
            self.0.iter_mut().for_each(|coord | coord.rotate_y());
        }
        for _ in 0..z {
            self.0.iter_mut().for_each(|coord | coord.rotate_z());
        }

        // Normalise
        let min_x = self.0.iter().map(|coord| coord.x).min().unwrap();
        let min_y = self.0.iter().map(|coord| coord.y).min().unwrap();
        let min_z = self.0.iter().map(|coord| coord.z).min().unwrap();
        
        self.0.iter_mut().for_each(|coord| coord.x = coord.x - min_x);
        self.0.iter_mut().for_each(|coord| coord.y = coord.y - min_y);
        self.0.iter_mut().for_each(|coord| coord.z = coord.z - min_z);
    }

    fn bitmask(&self) -> u64 {
        let mut mask: u64 = 0;
        for coord in &self.0 {
            mask |= 1<<((coord.z as u64)*16+(coord.y as u64)*4 + (coord.x as u64))
        }
        mask
    }
}

impl PuzzlePiece {
    fn new(name: String, base: Orientation) -> PuzzlePiece {
        PuzzlePiece { name, base, orientations: vec![], permutations: vec![] }
    }

    fn from_csv(path: PathBuf) -> Result<Vec<PuzzlePiece>, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut pieces = vec![];
        for result in rdr.records() {
            let record = result?;
            let colour = record[1].parse().unwrap_or(Color::BrightRed);
            pieces.push(PuzzlePiece::new(
                record[0].color(colour).to_string(),
                Orientation(Coord::from_str(&record[2])),
            ));
        }
        Ok(pieces)
    }

    fn generate_total_orientations(&mut self) {
        // Has six faces
        // Each face can be in four rotations
        // Good resource: https://www.euclideanspace.com/maths/geometry/rotations/euler/examples/index.htm
        //      Matrix rep: https://www.euclideanspace.com/maths/algebra/matrix/transforms/examples/index.htm
        let mut current_orientation = self.base.clone();
        let mut orientations: Vec<Orientation> = vec![];
        for _ in 0..4 {
            orientations.push(current_orientation.clone());
            let mut o = current_orientation.clone();
            o.rotate(0, 1, 0);
            orientations.push(o);
            let mut o = current_orientation.clone();
            o.rotate(0, 3, 0);
            orientations.push(o);
            let mut o = current_orientation.clone();
            o.rotate(0, 0, 1);
            orientations.push(o);
            let mut o = current_orientation.clone();
            o.rotate(0, 0, 2);
            orientations.push(o);
            let mut o = current_orientation.clone();
            o.rotate(0, 0, 3);
            orientations.push(o);

            current_orientation.rotate(1, 0, 0);
        }
        let unique_orientations: Vec<Orientation> = orientations.iter().unique().map(|x| x.clone()).collect();
        println!("{self:?}: {}", unique_orientations.len());
        self.orientations = unique_orientations;
    }

}

fn pretty_bitmask(mask: u64) {
    for y in 0..4 {
        for z in 0..4 {
           for x in 0..4 {
               if (((mask >> 16*z) >> 4*y) >> x)&1==1 {
                   print!("X");
               } else {
                   print!(".");
               }
           }
           print!(" ");
       }
       println!();
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    let mut pieces = PuzzlePiece::from_csv(PathBuf::from("pieces.csv"))?;

    println!("Orientations");

    for piece in pieces.iter_mut() {
        piece.generate_total_orientations();
    }

    println!("Positions");

    Ok(())
}
