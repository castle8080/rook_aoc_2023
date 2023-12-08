mod problems;
mod aocbase;
mod aocio;

#[macro_use]
mod run;

fn main() {
    run_problem!(problem1, part1, "input/input_01.txt");
    run_problem!(problem1, part2, "input/input_01.txt");
    run_problem!(problem2, part1, "input/input_02.txt");
    run_problem!(problem2, part2, "input/input_02.txt");
    run_problem!(problem3, part1, "input/input_03.txt");
    run_problem!(problem3, part2, "input/input_03.txt");
    run_problem!(problem4, part1, "input/input_04.txt");
    run_problem!(problem4, part2, "input/input_04.txt");
    run_problem!(problem5, part1, "input/input_05.txt");
    run_problem!(problem5, part2, "input/input_05.txt");
    run_problem!(problem6, part1, "input/input_06.txt");
    run_problem!(problem6, part2, "input/input_06.txt");
    run_problem!(problem7, part1, "input/input_07.txt");
    run_problem!(problem7, part2, "input/input_07.txt");
    run_problem!(problem8, part1, "input/input_08.txt");
    run_problem!(problem8, part2, "input/input_08.txt");
}