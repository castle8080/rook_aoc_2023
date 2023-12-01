use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;

fn get_line_num_value(line: &String) -> Option<i32> {
    let mut first_digit: Option<i32> = None;
    let mut last_digit: Option<i32> = None;

    for c in line.chars() {
        if c >= '0' && c <= '9' {
            let n = c as i32 - '0' as i32;
            if let None = first_digit {
                first_digit = Some(n);
            }
            last_digit = Some(n);
        }
    }

    match (first_digit, last_digit) {
        (Some(fd), Some(ld)) => {
            Some(fd * 10 + ld)
        },
        _ => None
    }
}

pub struct NumMatcher {
    pub match_value: Vec<char>,
    pub value: i32,
}

impl NumMatcher {
    pub fn new(s: impl AsRef<str>, value: i32) -> Self {
        NumMatcher {
            match_value: s.as_ref().chars().collect(),
            value: value
        }
    }

    pub fn is_match(&self, text: &[char]) -> bool {
        text.starts_with(&self.match_value)
    }
}

struct NumMatchers {
    matchers: Vec<NumMatcher>,
}

impl NumMatchers {
    pub fn default() -> NumMatchers {
        let mut matchers = vec![
            NumMatcher::new("one", 1),
            NumMatcher::new("two", 2),
            NumMatcher::new("three", 3),
            NumMatcher::new("four", 4),
            NumMatcher::new("five", 5),
            NumMatcher::new("six", 6),
            NumMatcher::new("seven", 7),
            NumMatcher::new("eight", 8),
            NumMatcher::new("nine", 9),
        ];

        for d in 0..=9 {
            matchers.push(NumMatcher::new(d.to_string(), d));
        }

        NumMatchers { matchers }
    }

    pub fn get_digit(&self, text: &[char]) -> Option<i32> {
        for m in &self.matchers {
            if m.is_match(text) {
                return Some(m.value);
            }
        }

        return None;
    }

    pub fn get_line_num(&self, line: &String) -> Option<i32> {
        let mut first_digit: Option<i32> = None;
        let mut last_digit: Option<i32> = None;

        let chars: Vec<char> = line.chars().collect();

        for n in 0..chars.len() {
            let cseq = &chars[n..];
            match self.get_digit(cseq) {
                Some(d) => {
                    if let None = first_digit {
                        first_digit = Some(d);
                    }
                    last_digit = Some(d);
                },
                None => {}
            }
        }

        match (first_digit, last_digit) {
            (Some(fd), Some(ld)) => {
                Some(fd * 10 + ld)
            },
            _ => None
        }
    }
}

pub fn part1(input: impl AsRef<Path>) -> Result<String, Box<dyn Error>> {
    let mut reader = BufReader::new(File::open(input)?);
    let mut buffer = String::new();
    let mut result = 0;

    while reader.read_line(&mut buffer)? > 0 {
        match get_line_num_value(&buffer) {
            Some(v) => {
                result += v;
            }
            None => {}
        }
        buffer.clear();
    }

    Ok(format!("{result}"))
}

pub fn part2(input: impl AsRef<Path>) -> Result<String, Box<dyn Error>> {
    let mut reader = BufReader::new(File::open(input)?);
    let mut buffer = String::new();
    let mut result = 0;

    let num_matchers = NumMatchers::default();

    while reader.read_line(&mut buffer)? > 0 {
        match num_matchers.get_line_num(&buffer) {
            Some(v) => {
                result += v;
            }
            None => {}
        }
        buffer.clear();
    }

    Ok(format!("{result}"))
}