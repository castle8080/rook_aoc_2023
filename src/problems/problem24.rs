use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::num::ParseFloatError;
use std::path::Path;

use lazy_static::lazy_static;
use regex::Regex;

use crate::aocbase::{AOCResult, AOCError};

lazy_static! {
    static ref HAIL_BALL_REGEX: Regex = Regex::new(r"[\s,@]+").unwrap();
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct HailBall {
    x: f64,
    y: f64,
    z: f64,
    xd: f64,
    yd: f64,
    zd: f64,
}

impl HailBall {

    pub fn parse(text: impl AsRef<str>) -> AOCResult<HailBall> {
        let nums = HAIL_BALL_REGEX
            .split(text.as_ref())
            .filter(|s| s.len() > 0)
            .map(|s| s.parse::<f64>())
            .collect::<Result<Vec<f64>, ParseFloatError>>()?;

        if nums.len() != 6 {
            return Err(AOCError::ParseError(format!("Invalid hail ball: {}", text.as_ref())))
        }

        Ok(HailBall {
            x: nums[0], y: nums[1], z: nums[2],
            xd: nums[3], yd: nums[4], zd: nums[5],
        })
    }

    pub fn parse_all(input: impl AsRef<Path>) -> AOCResult<Vec<HailBall>> {
        let reader = BufReader::new(File::open(input.as_ref())?);
        
        let mut hail_balls: Vec<HailBall> = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            hail_balls.push(Self::parse(line)?);
        }

        Ok(hail_balls)
    }

    pub fn xy_intersect(&self, other: &HailBall) -> Option<(f64, f64, f64, f64)> {

        // linear equation
        //   y = mx + b
        // 
        // m = yd/xd
        // b = y - mx
        //

        let m1 = self.yd / self.xd;
        let m2 = other.yd / other.xd;

        let b1 = self.y - m1 * self.x;
        let b2 = other.y - m2 * other.x;

        // m1 * X + b1 = m2 * X + b2
        // m1 * X = m2 * X + b2 - b1
        // m1 * X - m2 * X = b2 - b1
        // X (m1 - m2) = b2 - b1
        // X = (b2 - b1) / (m1 - m2)

        if m1 == m2 {
            return None;
        }

        let x = (b2 - b1) / (m1 - m2);
        let y = m1 * x + b1;

        // Figure out time
        // Xn = Xi + t*xd
        // (Xn - Xi) / xd = t

        let t1 = (x - self.x) / self.xd;
        let t2 = (x - other.x) / other.xd;

        Some((x, y, t1, t2))
    }
}

pub fn get_future_xy_crossings<'a>(
    hail_balls: &'a Vec<HailBall>,
    test_start: f64,
    test_end: f64) -> Vec<(&'a HailBall, &'a HailBall)>
{
    let mut crossings: Vec<(&'a HailBall, &'a HailBall)> = Vec::new();

    for i1 in 0 .. hail_balls.len() - 1 {
        for i2 in i1+1 .. hail_balls.len() {
            let hb1 = &hail_balls[i1];
            let hb2 = &hail_balls[i2];

            match hb1.xy_intersect(&hb2) {
                Some((x, y, t1, t2)) => {
                    if x >= test_start && x <= test_end &&
                        y >= test_start && y <= test_end &&
                        t1 >= 0.0 &&
                        t2 >= 0.0
                    {
                        crossings.push((hb1, hb2));
                    }
                },
                None => {}
            }
        }
    }

    crossings
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let hail_balls = HailBall::parse_all(input)?;

    //let crossings = get_future_xy_crossings(&hail_balls, 7.0, 27.0);
    let crossings = get_future_xy_crossings(&hail_balls, 200000000000000.0, 400000000000000.0);

    let result = crossings.len();

    Ok(result.to_string())
}