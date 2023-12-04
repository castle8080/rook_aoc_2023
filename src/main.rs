mod problems;
mod aocbase;
mod aocio;

use std::time::Instant;
use std::panic;

macro_rules! run_problem {
    // Match the pattern for the macro
    ($problem:ident, $part:ident, $input:expr) => {{
        use problems::$problem;
        println!("Running: {}::{}", stringify!($problem), stringify!($part));
        let start = Instant::now();

        let result = panic::catch_unwind(|| {
            $problem::$part($input)
        });

        let duration = start.elapsed();

        match result {
            Ok(Ok(answer)) => {
                println!("Answer: {}", answer);
            },
            Ok(Err(e)) => {
                println!("Failed: {:?}", e);
            },
            Err(e) => {
                println!("Failed: Panic {:?}", e);
            }
        }
        println!("Elapsed Time: {} milliseconds.", duration.as_micros() as f32 / 1000.0);
    }};
}

fn main() {
    run_problem!(problem1, part1, "input/input_01.txt");
    run_problem!(problem1, part2, "input/input_01.txt");
    run_problem!(problem2, part1, "input/input_02.txt");
    run_problem!(problem2, part2, "input/input_02.txt");
    run_problem!(problem3, part1, "input/input_03.txt");
    run_problem!(problem3, part2, "input/input_03.txt");
    run_problem!(problem4, part1, "input/input_04.txt");
    run_problem!(problem4, part2, "input/input_04.txt");
}
