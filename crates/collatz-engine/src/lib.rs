#![forbid(unsafe_code)]

//! Correctness-first scalar engines and generators for the standard Collatz map.

mod domain;
mod generators;
mod reference;

pub use domain::{
    ArithmeticOverflow, PositiveU128, PositiveU128Error, RunError, RunProgress, RunSummary,
    Termination,
};
pub use generators::{MersenneError, mersenne};
pub use reference::{run, step};
