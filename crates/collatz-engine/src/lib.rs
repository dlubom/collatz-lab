#![forbid(unsafe_code)]

//! Correctness-first scalar engines and generators for the standard Collatz map.

mod bigint;
mod domain;
mod generators;
mod hybrid;
mod reference;

pub use bigint::{bigint_step, run_bigint};
pub use domain::{
    ArithmeticOverflow, BigIntRunSummary, HybridRunSummary, HybridValue, PositiveInteger,
    PositiveIntegerError, PositiveU128, PositiveU128Error, RunError, RunProgress, RunSummary,
    Termination,
};
pub use generators::{MersenneError, mersenne};
pub use hybrid::run_hybrid;
pub use reference::{run, step};

#[cfg(test)]
mod tests {
    use std::error::Error as _;

    use super::*;

    fn positive(value: u128) -> PositiveU128 {
        PositiveU128::new(value).expect("unit-test literals are positive")
    }

    #[test]
    fn positive_domain_construction_and_conversions_are_typed() {
        assert_eq!(PositiveU128::new(0), Err(PositiveU128Error::Zero));
        assert_eq!(PositiveU128::try_from(7), Ok(positive(7)));
        assert_eq!(u128::from(positive(7)), 7);
        assert_eq!(
            PositiveU128Error::Zero.to_string(),
            "Collatz input must be positive"
        );
    }

    #[test]
    fn classical_step_covers_even_odd_and_both_overflow_operations() {
        assert_eq!(step(positive(6)), Ok(positive(3)));
        assert_eq!(step(positive(3)), Ok(positive(10)));

        let addition_overflow = step(positive(u128::MAX / 3)).expect_err("addition overflows");
        assert_eq!(addition_overflow.current, positive(u128::MAX / 3));
        assert!(addition_overflow.to_string().contains("not representable"));

        let multiplication_overflow = step(positive(u128::MAX)).expect_err("multiply overflows");
        assert_eq!(multiplication_overflow.current, positive(u128::MAX));
    }

    #[test]
    fn finite_runner_covers_terminal_limit_metrics_and_error_progress() {
        let terminal = run(positive(1), 0).expect("one is terminal before the limit check");
        assert_eq!(terminal.termination, Termination::ReachedOne);
        assert_eq!(terminal.completed_classical_steps, 0);
        assert_eq!(terminal.classical_step_limit, 0);

        let limited = run(positive(8), 1).expect("one even transition is representable");
        assert_eq!(limited.last, positive(4));
        assert_eq!(limited.observed_peak, positive(8));
        assert_eq!(limited.first_descent_step, Some(1));
        assert_eq!(limited.termination, Termination::StepLimitReached);

        let overflow = run(positive(u128::MAX / 3), 1).expect_err("first transition overflows");
        assert_eq!(overflow.progress.completed_classical_steps, 0);
        assert_eq!(overflow.progress.classical_step_limit, 1);
        assert_eq!(overflow.progress.last, positive(u128::MAX / 3));
        assert!(
            overflow
                .to_string()
                .contains("after 0 completed classical steps")
        );
        assert!(overflow.source().is_some());
    }

    #[test]
    fn mersenne_generator_covers_domain_and_representation_boundaries() {
        assert_eq!(mersenne(0), Err(MersenneError::ZeroExponent));
        assert_eq!(mersenne(5), Ok(positive(31)));
        assert_eq!(mersenne(128), Ok(positive(u128::MAX)));
        assert_eq!(
            mersenne(129),
            Err(MersenneError::ExponentTooLarge { exponent: 129 })
        );
        assert_eq!(
            MersenneError::ZeroExponent.to_string(),
            "Mersenne exponent must be at least 1"
        );
        assert!(
            MersenneError::ExponentTooLarge { exponent: 129 }
                .to_string()
                .contains("129")
        );
    }
}
