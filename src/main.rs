use std::thread::spawn;

use crate::puzzle::Puzzle;

mod puzzle;

fn main() {
    let puzzles = [
        Puzzle::number(66),
        Puzzle::number(66),
    ];

    let mut handles = vec![];

    for puzzle in puzzles {
        let handle = spawn(move || puzzle.compute(666));
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
