use crate::aocbase::{AOCResult, AOCError};

use std::collections::HashMap;
use std::path::Path;
use std::fs::create_dir_all;
use std::time::{Instant, Duration};
use regex::Regex;

use crate::regex_ext::CapturesExt;
use crate::regex_ext::RegexExt;

pub struct Problem {
    pub name: String,
    pub runner: Box<dyn Fn(&String) -> AOCResult<String>>,
}

pub struct ProblemResult {
    pub name: String,
    pub start: Instant,
    pub duration: Duration,
    pub result: AOCResult<String>,
}

impl ProblemResult {

    pub fn get_duration_ms(&self) -> f64 {
        self.duration.as_micros() as f64 / 1000.0
    }

    pub fn to_stdout(&self) {
        println!("Finished: {}", &self.name);
        println!("Duration: {} milliseconds", self.get_duration_ms());
        match &self.result {
            Ok(answer) => {
                println!("Answer: {}", answer);
            },
            Err(e) => {
                println!("Failed: {:?}", e);
            }
        }
    }
}

pub struct ProblemResults {
}

impl ProblemResults {

    pub fn load_answers(csv_path: impl AsRef<Path>) -> AOCResult<HashMap<String, String>> {
        let csv_path = csv_path.as_ref();

        if !csv_path.is_file() {
            return Ok(HashMap::new());
        }

        let mut answers: HashMap<String, String> = HashMap::new();

        let mut csv_in = csv::Reader::from_path(&csv_path)?;
        for record in csv_in.deserialize() {
            let record: HashMap<String, String> = record?;

            let problem = record
                .get("Problem")
                .ok_or(AOCError::ParseError("Problem field not present.".into()))?;

            let answer = record
                .get("Answer")
                .ok_or(AOCError::ParseError("Answer field not present.".into()))?;

            answers.insert(problem.into(), answer.into());
        }

        Ok(answers)
    }

    pub fn write_csv(path: impl AsRef<Path>, results: &Vec<ProblemResult>) -> AOCResult<()> {
        let path = path.as_ref();

        // Make sure the parent directory exists.
        if let Some(parent) = path.parent() {
            create_dir_all(&parent)?;
        }

        let mut csv_out = csv::Writer::from_path(path)?;

        csv_out.write_record(vec!["Problem", "Duration", "Answer", "Error"])?;

        for result in results {
            match &result.result {
                Ok(answer) => {
                    csv_out.write_record(vec![
                        result.name.clone(),
                        result.get_duration_ms().to_string(),
                        answer.into(),
                        "".into()
                    ])?;
                },
                Err(e) => {
                    csv_out.write_record(vec![
                        result.name.clone(),
                        result.get_duration_ms().to_string(),
                        "".into(),
                        e.to_string(),
                    ])?;
                }
            }
        }

        Ok(())
    }

}

impl Problem {

    pub fn run(&self, input: &String) -> ProblemResult {
        println!("--------------------------------------");
        println!("Starting: {}", self.name);
        let start = Instant::now();
        let result = (self.runner)(input);
        let duration = start.elapsed();
        ProblemResult {
            name: self.name.clone(),
            start,
            duration,
            result
        }
    }

    pub fn get_default_input(&self) -> AOCResult<String> {
        let p_num = parse_number(&self.name)?;
        Ok(format!("input/input_{:0>2}.txt", p_num).into())
    }
}

pub fn parse_number(name: impl AsRef<str>) -> AOCResult<i32> {
    Ok(Regex::new(r"(\d+)")?
        .captures_must(name.as_ref())?
        .get_group(1)?
        .parse::<i32>()?)
}

macro_rules! problems {
    [$($problem:ident::$part:ident,)*] => {
        vec![$(problem!($problem::$part),)*]
    }
}

macro_rules! problem {
    ($problem:ident::$part:ident) => {{
        use problems::$problem;
        use crate::run::Problem;

        let name = format!("{}::{}", stringify!($problem), stringify!($part)).to_string();
        Problem {
            name: name,
            runner: Box::new(|input: &String| $problem::$part(input))
        }
    }}
}