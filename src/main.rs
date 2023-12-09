mod problems;
mod aocbase;
mod aocio;

#[macro_use]
mod run;

fn main() {
    run::run_args(problems![
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
    ]);
}