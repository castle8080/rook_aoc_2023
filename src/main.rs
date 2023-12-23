mod problems;
mod aocbase;
mod aocio;
mod regex_ext;
mod mathx;

#[macro_use]
mod run;

use clap::Parser;

use run::{Problem, ProblemResult, ProblemResults};
use aocbase::AOCResult;

const DEFAULT_RESULT_FILE: &str = "results/latest.csv";
const DEFAULT_LAST_RESULT_FILE: &str = "results/last.csv";

fn get_problems() -> Vec<Problem> {
    problems![
        problem1::part1,
        problem1::part2,
        problem2::part1,
        problem2::part2,
        problem3::part1,
        problem3::part2,
        problem4::part1,
        problem4::part2,
        problem5::part1,
        problem5::part2,
        problem6::part1,
        problem6::part2,
        problem7::part1,
        problem7::part2,
        problem8::part1,
        problem8::part2,
        problem9::part1,
        problem9::part2,
        problem10::part1,
        problem10::part2,
        problem11::part1,
        problem11::part2,
        problem12::part1,
        problem12::part2,
        problem13::part1,
        problem13::part2,
        problem14::part1,
        problem14::part2,
        problem15::part1,
        problem15::part2,
        problem16::part1,
        problem16::part2,
        problem17::part1,
        problem17::part2,
        problem18::part1,
        problem18::part2,
        problem19::part1,
        problem19::part2,
        problem20::part1,
        problem20::part2,
        problem21::part1,
        problem21::part2,
        problem22::part1,
        problem22::part2,
    ]
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, short)]
    problem: Option<String>,

    #[arg(long, short)]
    input: Option<String>,

    #[arg(long, short)]
    result_file: Option<String>,

    #[arg(long, short)]
    last_result_file: Option<String>,
}

impl Args {

    pub fn get_result_file<'a>(&'a self) -> &str {
        match &self.result_file {
            None => DEFAULT_RESULT_FILE,
            Some(result_file) => result_file.as_str(),
        }
    }

    pub fn get_last_result_file<'a>(&'a self) -> &str {
        match &self.last_result_file {
            None => DEFAULT_LAST_RESULT_FILE,
            Some(result_file) => result_file.as_str(),
        }
    }

    pub fn compare_with_last(&self, results: &Vec<ProblemResult>) -> AOCResult<()> {
        let last_results = ProblemResults::load_answers(self.get_last_result_file())?;

        for result in results {
            match (&result.result, last_results.get(&result.name)) {
                (Ok(answer), None) => {
                    println!("New Answer: [{}] {}", &result.name, answer);
                },
                (Ok(answer), Some(last_answer)) if answer != last_answer => {
                    println!("Mismatch: [{}] {} != {}", &result.name, last_answer, answer);
                },
                (Err(e), Some(last_answer)) if last_answer != "" => {
                    println!("Mismatch: [{}] {} != {}", &result.name, last_answer, e);
                },
                _ => {}
            }
        }

        Ok(())
    }

    pub fn run(&self) -> AOCResult<()> {
        let problems = get_problems();
    
        let to_run: Vec<&Problem> = match &self.problem {
            None => problems.iter().collect(),
            Some(problem) => problems.iter().filter(|p| &p.name == problem).collect(),
        };
    
        if to_run.len() == 0 {
            panic!("There were no matching problems found to run!");
        }
    
        let mut results: Vec<ProblemResult> = Vec::new();

        for p in to_run {
            let result = match &self.input {
                None => {
                    let input = p.get_default_input()?;
                    p.run(&input)
                },
                Some(input) => {
                    p.run(input)
                }
            };

            result.to_stdout();
            results.push(result);
        }
    
        // Write results to file
        ProblemResults::write_csv(self.get_result_file(), &results)?;

        // Show if there are any differences from a previous run.
        self.compare_with_last(&results)?;
        Ok(())
    }

}

fn main() {
    let args = Args::parse();
    args.run().unwrap();
}