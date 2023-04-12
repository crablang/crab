//! Helper module which helps to determine amount of threads to be used
//! during tests execution.
use std::{env, num::NonZeroUsize, thread};

pub fn get_concurrency() -> usize {
    if let Ok(value) = env::var("CRABLANG_TEST_THREADS") {
        match value.parse::<NonZeroUsize>().ok() {
            Some(n) => n.get(),
            _ => panic!("CRABLANG_TEST_THREADS is `{value}`, should be a positive integer."),
        }
    } else {
        thread::available_parallelism().map(|n| n.get()).unwrap_or(1)
    }
}
