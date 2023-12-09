mod problems;
mod aocbase;
mod aocio;

#[macro_use]
mod run;

fn main() {
    let problems = vec![
        problem!(problem1, part1, "input/input_01.txt"),
        problem!(problem1, part2, "input/input_01.txt"),
        problem!(problem2, part1, "input/input_02.txt"),
        problem!(problem2, part2, "input/input_02.txt"),
        problem!(problem3, part1, "input/input_03.txt"),
        problem!(problem3, part2, "input/input_03.txt"),
        problem!(problem4, part1, "input/input_04.txt"),
        problem!(problem4, part2, "input/input_04.txt"),
        problem!(problem5, part1, "input/input_05.txt"),
        problem!(problem5, part2, "input/input_05.txt"),
        problem!(problem6, part1, "input/input_06.txt"),
        problem!(problem6, part2, "input/input_06.txt"),
        problem!(problem7, part1, "input/input_07.txt"),
        problem!(problem7, part2, "input/input_07.txt"),
        problem!(problem8, part1, "input/input_08.txt"),
        problem!(problem8, part2, "input/input_08.txt"),
        problem!(problem9, part1, "input/input_09.txt"),
        problem!(problem9, part2, "input/input_09.txt"),
    ];

    let args: Vec<String> = std::env::args().skip(1).collect();
    run::run_problems(&problems, &args);
}