use crate::engine::Engine;
use crate::score::Score;
use std::time::{Duration, Instant};

pub struct Benchmark {
    pub score: Score,
    pub duration: Duration,
    pub work_count: usize,
    pub runs: usize,
}

impl Benchmark {
    pub fn run(engine: &mut Engine) -> Benchmark {
        let start_work = engine.work_count;
        let start_time = Instant::now();
        let score = engine.solve();
        let duration = start_time.elapsed();
        let work_count = engine.work_count - start_work;

        Benchmark {
            score,
            duration,
            work_count,
            runs: 1,
        }
    }

    pub fn empty() -> Benchmark {
        Benchmark {
            score: Score::Unknown,
            duration: Duration::from_secs(0),
            work_count: 0,
            runs: 0,
        }
    }

    pub fn add(&self, other: &Benchmark) -> Benchmark {
        Benchmark {
            score: self.score,
            duration: self.duration + other.duration,
            work_count: self.work_count + other.work_count,
            runs: self.runs + other.runs,
        }
    }

    pub fn get_speed(&self) -> f64 {
        self.work_count as f64 / self.duration.as_secs_f64()
    }

    pub fn print(&self) {
        let width = 6;
        if self.runs == 1 {
            println!("The score is {:?}", self.score);
        }
        println!(
            "Total time: {:>width$.3} s",
            self.duration.as_secs_f64(),
            width = width
        );
        println!(
            "Total work: {}",
            format_large_number(self.work_count as f64, width)
        );
        println!(
            "Speed:      {}/s",
            format_large_number(self.get_speed(), width)
        );
    }
}

pub fn format_large_number(n: f64, width: usize) -> String {
    if n < 100_000.0 {
        format!("{:>width$}", n, width = width)
    } else {
        format!("{:>width$.3} M", n / 1_000_000.0, width = width)
    }
}
