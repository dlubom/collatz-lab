use collatz_engine::{PositiveU128, RunError, RunProgress, RunSummary, Termination, run, step};
use proptest::prelude::*;

fn positive(value: u128) -> PositiveU128 {
    PositiveU128::new(value).expect("generated test inputs are positive")
}

fn expected_prefix(start: PositiveU128, limit: u64) -> Result<RunSummary, RunError> {
    let mut current = start;
    let mut completed_classical_steps = 0;
    let mut observed_peak = start;
    let mut first_descent_step = None;

    loop {
        if current.get() == 1 {
            return Ok(RunSummary {
                start,
                last: current,
                completed_classical_steps,
                classical_step_limit: limit,
                observed_peak,
                first_descent_step,
                termination: Termination::ReachedOne,
            });
        }

        if completed_classical_steps == limit {
            return Ok(RunSummary {
                start,
                last: current,
                completed_classical_steps,
                classical_step_limit: limit,
                observed_peak,
                first_descent_step,
                termination: Termination::StepLimitReached,
            });
        }

        let next = match step(current) {
            Ok(next) => next,
            Err(overflow) => {
                return Err(RunError {
                    overflow,
                    progress: RunProgress {
                        start,
                        last: current,
                        completed_classical_steps,
                        classical_step_limit: limit,
                        observed_peak,
                        first_descent_step,
                    },
                });
            }
        };

        completed_classical_steps += 1;
        current = next;
        observed_peak = observed_peak.max(current);
        if first_descent_step.is_none() && current < start {
            first_descent_step = Some(completed_classical_steps);
        }
    }
}

proptest! {
    #[test]
    fn even_branch_divides_a_positive_value_by_two(half in 1_u128..=u128::MAX / 2) {
        let current = positive(half * 2);
        let next = step(current).expect("halving a positive u128 is representable");

        prop_assert_eq!(next.get(), half);
        prop_assert!(next.get() > 0);
    }

    #[test]
    fn representable_odd_branch_uses_three_n_plus_one(
        odd_index in 0_u128..=(u128::MAX - 7) / 6,
    ) {
        let value = odd_index * 2 + 1;
        let next = step(positive(value)).expect("generated odd step is representable");

        prop_assert_eq!(next.get(), value * 3 + 1);
        prop_assert_eq!(next.get() % 2, 0);
        prop_assert!(next.get() > 0);
    }

    #[test]
    fn finite_runs_match_the_public_single_step_prefix(
        start in 1_u128..=1_000_000,
        limit in 0_u64..=512,
    ) {
        let start = positive(start);

        prop_assert_eq!(run(start, limit), expected_prefix(start, limit));
    }

    #[test]
    fn successful_run_metrics_respect_basic_bounds(
        start in 1_u128..=1_000_000,
        limit in 0_u64..=512,
    ) {
        let start = positive(start);
        let summary = run(start, limit).expect("small generated prefixes stay in u128");

        prop_assert!(summary.completed_classical_steps <= limit);
        prop_assert_eq!(summary.classical_step_limit, limit);
        prop_assert!(summary.observed_peak >= start);
        prop_assert!(summary.observed_peak >= summary.last);
        if let Some(first_descent_step) = summary.first_descent_step {
            prop_assert!(first_descent_step >= 1);
            prop_assert!(first_descent_step <= summary.completed_classical_steps);
        }
        if summary.termination == Termination::ReachedOne {
            prop_assert_eq!(summary.last.get(), 1);
        } else {
            prop_assert_eq!(summary.completed_classical_steps, limit);
            prop_assert_ne!(summary.last.get(), 1);
        }
    }
}
