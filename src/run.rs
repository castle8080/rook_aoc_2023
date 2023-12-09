use crate::aocbase::{AOCResult, AOCError};

use std::time::Instant;
use regex::Regex;

pub struct Problem {
    pub name: String,
    pub input: String,
    pub runner: Box<dyn Fn(&String) -> AOCResult<String>>,
}

impl Problem {

    pub fn run(&self, input: &String) {
        println!("Running: {}", self.name);
        let start = Instant::now();

        let result = (self.runner)(input);

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

pub fn parse_number(name: impl AsRef<str>) -> AOCResult<i32> {
    Ok(Regex::new(r"(\d+)")?
        .captures(name.as_ref())
        .ok_or_else(|| AOCError::ParseError("Name has no number.".into()))?
        .get(1)
        .ok_or_else(|| AOCError::InvalidRegexOperation("Invalid group".into()))?
        .as_str()
        .parse::<i32>()?)
}

macro_rules! problems {
    [$($problem:ident::$part:ident,)*] => {
        vec![$(problem!($problem::$part),)*]
    }
}

macro_rules! problem {
    // Match the pattern for the macro
    ($problem:ident::$part:ident) => {{
        use problems::$problem;
        use crate::run::{Problem, parse_number};

        let name = format!("{}::{}", stringify!($problem), stringify!($part)).to_string();
        let problem_n = parse_number(&name).unwrap();
        let default_input = format!("input/input_{:0>2}.txt", problem_n);

        Problem {
            name: name,
            input: default_input.to_string(),
            runner: Box::new(|input: &String| $problem::$part(input))
        }
    }}
}

/// run_args
pub fn run_args(problems: Vec<Problem>) {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.len() == 0 {
        for p in problems {
            p.run(&p.input);
        }
    }
    else if args.len() == 2 {
        match problems.iter().find(|p| p.name == args[0]) {
            None => {
                panic!("ERROR: Problem: {} does not exist.", args[0]);
            },
            Some(p) => {
                p.run(&args[1]);
            }
        }
    }
    else {
        panic!("EROR: Usage <problem> <input>")
    }
}