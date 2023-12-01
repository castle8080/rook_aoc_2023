mod problems;

use crate::problems::problem1;
use std::time::Instant;

macro_rules! run_problem {
    // Match the pattern for the macro
    ($problem:ident, $part:ident, $input:expr) => {
        println!("Running: {}::{}", stringify!($problem), stringify!($part));
        let start = Instant::now();
        let result = $problem::$part($input);
        let duration = start.elapsed();

        match result {
            Ok(answer) => {
                println!("Answer: {}", answer);
            },
            Err(e) => {
                println!("Failed: {:?}", e);
            }
        }
        println!("Elapsed Time: {} milliseconds.", duration.as_micros() as f32 / 1000.0);
    };
}

fn main() {
    run_problem!(problem1, part1, "input/input_01.txt");
    run_problem!(problem1, part2, "input/input_01.txt");
}
