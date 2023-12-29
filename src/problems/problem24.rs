use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::num::ParseFloatError;
use std::path::Path;

use bigdecimal::FromPrimitive;
use bigdecimal::ToPrimitive;
use bigdecimal::Zero;
use lazy_static::lazy_static;
use regex::Regex;
use bigdecimal::BigDecimal;

use crate::aocbase::{AOCResult, AOCError};

lazy_static! {
    static ref HAIL_BALL_REGEX: Regex = Regex::new(r"[\s,@]+").unwrap();
    static ref NEAR_ZERO: BigDecimal = BigDecimal::from_f64(0.000001).unwrap();
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HailBall {
    x: f64,
    y: f64,
    z: f64,
    xv: f64,
    yv: f64,
    zv: f64,
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
            xv: nums[3], yv: nums[4], zv: nums[5],
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
        // m = yv/xv
        // b = y - mx
        //

        let m1 = self.yv / self.xv;
        let m2 = other.yv / other.xv;

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
        // Xn = Xi + t*xv
        // (Xn - Xi) / xv = t

        let t1 = (x - self.x) / self.xv;
        let t2 = (x - other.x) / other.xv;

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

pub struct HailBallIntersectSolverLR<'a> {
    hail_balls: &'a Vec<HailBall>,
    full_combinations: bool,
}

/* ----------------------------------------------------------------------------
    * Find an equation which relates 2 other balls to the intial ball.
    * The equation is only in terms of the x and y initial positions and
    * velocities. Z can be determined from those.
    *
    *     x0 + xv0 * tn = xn + xvn * tn
    *     tn * xv0 - tn * xvn = xn - x0
    *     tn * (xv0 - xvn) = xn - x0
    *    
    *            (xn - x0)
    *     tn =  --------------
    *            (xv0 - xvn)
    *    
    *    
    *            (yn - y0)
    *     tn =  --------------
    *            (yv0 - yvn)
    *    
    *    
    *     (xn - x0) * (yv0 - yvn) = (yn - y0) * (xv0 - xvn)
    *     
    *     xn * yv0 - xn * yvn - x0 * yv0 + x0 * yvn = yn * xv0 - yn * xvn - y0 * xv0 + y0 * xvn
    *     
    *       xn * yv0 
    *     - xn * yvn
    *     - x0 * yv0
    *     + x0 * yvn
    *          =
    *       yn * xv0
    *     - yn * xvn
    *     - y0 * xv0
    *     + y0 * xvn
    *     
    *    
    *     Move terms on 0 to 1 side
    *    
    *     - x0 * yv0
    *       y0 * xv0
    *          =
    *     - xn * yv0 
    *       xn * yvn
    *     - x0 * yvn
    *       yn * xv0
    *     - yn * xvn
    *     + y0 * xvn
    *    
    *     Bring in another ball and you can have a similar equation where the left hand sides are the same.
    *    
    *     - x0 * yv0
    *       y0 * xv0
    *          =
    *     - xm * yv0 
    *       xm * yvm
    *     - x0 * yvm
    *       ym * xv0
    *     - ym * xvm
    *     + y0 * xvm
    *    
    *     Set the equal to each other
    *    
    *     - xn * yv0 
    *       xn * yvn
    *     - x0 * yvn
    *       yn * xv0
    *     - yn * xvn
    *     + y0 * xvn
    *          =
    *     - xm * yv0 
    *       xm * yvm
    *     - x0 * yvm
    *       ym * xv0
    *     - ym * xvm
    *     + y0 * xvm
    *    
    *     Rearrange again:
    *    
    *     - x0 * yvn + x0 * yvm
    *       y0 * xvn - y0 * xvm
    *       yn * xv0 - ym * xv0
    *     - xn * yv0 + xm * yv0
    *             =
    *        - xn * yvn
    *          yn * xvn
    *          xm * yvm
    *        - ym * xvm
    *    
    *     And again
    *    
    *       x0  * (-yvn + yvm)
    *       y0  * (xvn - xvm)
    *      xv0  * (yn - ym)
    *      yv0  * (-xn + xm)
    *             =
    *        - xn * yvn
    *          yn * xvn
    *          xm * yvm
    *        - ym * xvm
    *
-----------------------------------------------------------------------------*/
impl<'a> HailBallIntersectSolverLR<'a> {
    fn new(hail_balls: &'a Vec<HailBall>, full_combinations: bool) -> Self {
        Self { hail_balls, full_combinations }
    }

    pub fn solve(&self) -> AOCResult<HailBall> {
        //
        // I am bummed that I had to move this to BigDecimal.
        // the precision was not working right for f64 and BigDecimal is
        // so much slower. I know I could optimize out to 1/3 by not resolving
        // for each variable on it's own, but I think finding a nother method
        // that using multiplication ellimnation for a systme of equations would
        // be better. I had orginally tried using linear regression and found
        // it wasn't correct either. The answer was close, but I didn't know that
        // at the time. I only determined what is really going on by plugging
        // into Wolfram Alpha some intermediate equations to see the precise answer.
        // I think I could move this to integer arithemetic if I really wanted too.
        //
        let matrix = self.build_equation_matrix();

        let x  = self.solve_variable(&matrix, "x", "r")?.to_f64().unwrap().round();
        let xv = self.solve_variable(&matrix, "xv", "r")?.to_f64().unwrap().round();
        let y  = self.solve_variable(&matrix, "y", "r")?.to_f64().unwrap().round();
        let yv = self.solve_variable(&matrix, "yv", "r")?.to_f64().unwrap().round();

        // Get time of x hit
        // t = (x - b1.x) / (b1.xv - xv)
        
        let b1 = &self.hail_balls[0];
        let b2 = &self.hail_balls[1];
        
        let t1 = (x - b1.x) / (b1.xv - xv);
        let t2 = (x - b2.x) / (b2.xv - xv);
        
        let z1 = b1.z + b1.zv * t1;
        let z2 = b2.z + b2.zv * t2;
        
        let zv = (z2 - z1) / (t2 - t1);

        // z + zv * t1 = z1
        // z = z1 - zv * t1
        let z = z1 - zv * t1;

        Ok(HailBall{ x, y, z, xv, yv, zv })
    }

    fn solve_variable(&self, matrix: &HashMap<String, Vec<BigDecimal>>, solve_var: &str, result_var: &str)
        -> AOCResult<BigDecimal>
    {
        let mut reduced_matrix = matrix.clone();
        
        for k in matrix.keys() {
            if k != solve_var && k != result_var {
                reduced_matrix = self.elliminate_variable(&reduced_matrix, k);
            }
        }

        let values = reduced_matrix[solve_var]
            .iter()
            .zip(&reduced_matrix[result_var])
            .filter(|(var_val, result_val)| &var_val.abs() >= &NEAR_ZERO && *result_val >= &NEAR_ZERO)
            .map(|(var_val, result_val)| result_val / var_val)
            .collect::<Vec<BigDecimal>>();

        if values.len() == 0 {
            return Err(AOCError::ProcessingError(format!("Unable to solve for: {}", solve_var)));
        }

        let values_len: BigDecimal = (values.len() as i64).try_into().unwrap();
        let result: BigDecimal = values.iter().fold(BigDecimal::zero(), |a, b| a + b) / values_len;

        Ok(result)
    }

    fn elliminate_variable(&self, matrix: &HashMap<String, Vec<BigDecimal>>, var_name: &str)
        -> HashMap<String, Vec<BigDecimal>>
    {
        // Use the multiplication method to get rid of a uknown for 2 equations equalling each other.
        let keys = matrix.keys().filter(|k| *k != var_name).collect::<Vec<&String>>();
        let len = matrix[var_name].len();

        let mut new_matrix: HashMap<String, Vec<BigDecimal>> = HashMap::new();
        for k in &keys {
            new_matrix.insert((*k).into(), Vec::new());
        }

        for i in 0 .. len - 1 {
            // left_var * m = right_var;
            // left_var = right_var / m
            // m = right_var / left_var

            let left_var = &matrix[var_name][i];

            // We can generate more combinations if data is low.
            let end_iter = if self.full_combinations { len } else { i + 2};

            for j in i+1 .. end_iter {
                let right_var = &matrix[var_name][j];
    
                if &left_var.abs() <= &NEAR_ZERO || &right_var.abs() <= &NEAR_ZERO {
                    continue;
                }
    
                let multiplier = right_var / left_var;
    
                for k in &keys {
                    let l_val = &matrix[*k][i];
                    let r_val = &matrix[*k][j];
                    let new_val = l_val - r_val / &multiplier;
                    new_matrix.get_mut(*k).unwrap().push(new_val);
                }
            }
        }

        new_matrix
    }

    fn build_equation_matrix(&self) -> HashMap<String, Vec<BigDecimal>> {
        let mut matrix: HashMap<String, Vec<BigDecimal>> = HashMap::new();

        let mut x_vec: Vec<BigDecimal> = Vec::new();
        let mut y_vec: Vec<BigDecimal> = Vec::new();
        let mut xv_vec: Vec<BigDecimal> = Vec::new();
        let mut yv_vec: Vec<BigDecimal> = Vec::new();
        let mut r_vec: Vec<BigDecimal> = Vec::new();

        for i in 0 .. self.hail_balls.len() - 1 {
            let b1 = &self.hail_balls[i];

            // We can generate more combinations if data is low.
            let end_iter = if self.full_combinations { self.hail_balls.len() } else { i + 2};

            for j in i+1 .. end_iter {
                let b2 = &self.hail_balls[j];

                x_vec.push((-b1.yv + b2.yv).try_into().unwrap());
                y_vec.push((b1.xv - b2.xv).try_into().unwrap());
                xv_vec.push((b1.y - b2.y).try_into().unwrap());
                yv_vec.push((-b1.x + b2.x).try_into().unwrap());

                r_vec.push((
                    -b1.x * b1.yv +
                    b1.y * b1.xv +
                    b2.x * b2.yv -
                    b2.y * b2.xv
                ).try_into().unwrap());
            }
        }

        matrix.insert("x".into(), x_vec);
        matrix.insert("y".into(), y_vec);
        matrix.insert("xv".into(), xv_vec);
        matrix.insert("yv".into(), yv_vec);
        matrix.insert("r".into(), r_vec);

        matrix
    }
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let hail_balls = HailBall::parse_all(input)?;

    //let crossings = get_future_xy_crossings(&hail_balls, 7.0, 27.0);
    let crossings = get_future_xy_crossings(&hail_balls, 200000000000000.0, 400000000000000.0);

    let result = crossings.len();

    Ok(result.to_string())
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    let hail_balls = HailBall::parse_all(input)?;
    let solver = HailBallIntersectSolverLR::new(&hail_balls, false);
    let b = solver.solve()?;
    let result = b.x + b.y + b.z;

    Ok(result.to_string())
}