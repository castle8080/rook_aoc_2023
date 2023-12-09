use std::num::ParseIntError;
use std::path::Path;

use crate::aocbase::AOCResult;
use crate::aocio::process_lines;

pub fn parse_line(line: impl AsRef<str>) -> AOCResult<Vec<i64>> {
    Ok(line.as_ref()
        .split_ascii_whitespace()
        .filter(|s| s.len() > 0)
        .map(|s| s.parse::<i64>())
        .collect::<Result<Vec<i64>, ParseIntError>>()?)
}

pub struct NumStack {
    nums: Vec<Vec<i64>>,
}

impl NumStack {

    pub fn new(initial: Vec<i64>) -> Self {
        let mut nums: Vec<Vec<i64>> = vec![initial];

        loop {
            let last_layer = &nums[nums.len() - 1];
            if NumStack::is_end_layer(last_layer) {
                return NumStack { nums };
            }
            nums.push(NumStack::next_layer(last_layer));
        }
    }

    pub fn extrapolate_next(&self) -> i64 {
        self.extrapolate(|a, cur| a[a.len() - 1] + cur)
    }

    pub fn extrapolate_prev(&self) -> i64 {
        self.extrapolate(|a, cur| a[0] - cur)
    }

    fn extrapolate<F>(&self, f: F) -> i64
        where F: Fn(&Vec<i64>, i64) -> i64
    {
        let mut cur: i64 = 0;

        for depth in (0 .. self.nums.len() - 1).rev() {
            let a = &self.nums[depth];
            cur = f(a, cur);
        }

        cur
    }

    fn next_layer(layer: &Vec<i64>) -> Vec<i64> {
        (0..layer.len()-1)
            .map(|idx| layer[idx+1] - layer[idx])
            .collect()
    }

    fn is_end_layer(layer: &Vec<i64>) -> bool {
        layer.len() <= 1 || layer.iter().all(|n| *n == 0)
    }
}

fn run_part<F>(input: impl AsRef<Path>, f: F) -> AOCResult<String>
    where F: Fn(&NumStack) -> i64
{
    let mut result: i64 = 0;

    process_lines(input, |line| {
        let nums = parse_line(line)?;
        let num_stack = NumStack::new(nums);
        result += f(&num_stack);
        Ok(())
    })?;

    Ok(result.to_string())
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    run_part(input, |num_stack| num_stack.extrapolate_next())
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    run_part(input, |num_stack| num_stack.extrapolate_prev())
}