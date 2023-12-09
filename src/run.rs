use crate::aocbase::AOCResult;

use std::time::Instant;

pub struct Problem {
    pub name: String,
    pub input: String,
    pub runner: Box<dyn Fn(&String) -> AOCResult<String>>,
}

impl Problem {

    pub fn run(&self) {
        println!("Running: {}", self.name);
        let start = Instant::now();

        let result = (self.runner)(&self.input);

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
    }
}

macro_rules! problem {
    // Match the pattern for the macro
    ($problem:ident, $part:ident, $input:expr) => {{
        use problems::$problem;
        use crate::run::Problem;

        let name = format!("{}::{}", stringify!($problem), stringify!($part)).to_string();

        Problem {
            name: name,
            input: $input.to_string(),
            runner: Box::new(|input: &String| $problem::$part(input))
        }
    }}
}

/// Run the problems where the names have a match in args.
/// If there are no args, all problems are assumed to be run.
pub fn run_problems(problems: &Vec<Problem>, args: &Vec<String>) {
    let mut to_run: Vec<&Problem> = Vec::new();

    if args.len() == 0 {
        to_run.extend(problems.iter());
    }
    else {
        for arg in args.iter() {
            to_run.extend(problems.iter().filter(|p| p.name.contains(arg)));
        }
    }

    for p in to_run {
        p.run();
    }
}