
// Some math functions to use with aoc.

// greatest common divisor
pub fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a.abs()
}

// least common multiple
pub fn lcm(a: i64, b: i64) -> i64 {
    a * b / gcd(a, b)
}
