
macro_rules! run_problem {
    // Match the pattern for the macro
    ($problem:ident, $part:ident, $input:expr) => {{
        use problems::$problem;
        use std::time::Instant;
        use std::panic;
        println!("Running: {}::{}", stringify!($problem), stringify!($part));
        let start = Instant::now();

        let result = panic::catch_unwind(|| {
            $problem::$part($input)
        });

        let duration = start.elapsed();

        match result {
            Ok(Ok(answer)) => {
                println!("Answer: {}", answer);
            },
            Ok(Err(e)) => {
                println!("Failed: {:?}", e);
            },
            Err(e) => {
                println!("Failed: Panic {:?}", e);
            }
        }
        println!("Elapsed Time: {} milliseconds.", duration.as_micros() as f32 / 1000.0);
    }};
}