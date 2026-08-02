use crate::{BigIntRunSummary, PositiveInteger, Termination};

/// Applies one exact standard, unaccelerated Collatz transition.
///
/// The result remains a `rug::Integer` through the complete operation and is
/// never narrowed to a bounded primitive.
pub fn bigint_step(current: &PositiveInteger) -> PositiveInteger {
    let mut next = current.get().clone();

    if next.is_even() {
        next >>= 1;
    } else {
        next *= 3;
        next += 1;
    }

    PositiveInteger(next)
}

/// Evaluates a finite exact trajectory with terminal-before-limit ordering.
pub fn run_bigint(start: PositiveInteger, classical_step_limit: u64) -> BigIntRunSummary {
    let mut current = start.clone();
    let mut completed_classical_steps = 0;
    let mut observed_peak = start.clone();
    let mut first_descent_step = None;

    loop {
        if current.get() == &1 {
            return summary(
                start,
                current,
                completed_classical_steps,
                classical_step_limit,
                observed_peak,
                first_descent_step,
                Termination::ReachedOne,
            );
        }

        if completed_classical_steps == classical_step_limit {
            return summary(
                start,
                current,
                completed_classical_steps,
                classical_step_limit,
                observed_peak,
                first_descent_step,
                Termination::StepLimitReached,
            );
        }

        current = bigint_step(&current);
        completed_classical_steps += 1;

        if current > observed_peak {
            observed_peak = current.clone();
        }
        if first_descent_step.is_none() && current < start {
            first_descent_step = Some(completed_classical_steps);
        }
    }
}

fn summary(
    start: PositiveInteger,
    last: PositiveInteger,
    completed_classical_steps: u64,
    classical_step_limit: u64,
    observed_peak: PositiveInteger,
    first_descent_step: Option<u64>,
    termination: Termination,
) -> BigIntRunSummary {
    BigIntRunSummary {
        start,
        last,
        completed_classical_steps,
        classical_step_limit,
        observed_peak,
        first_descent_step,
        termination,
    }
}

#[cfg(test)]
mod tests {
    use rug::Integer;

    use super::*;

    fn exact(value: u128) -> PositiveInteger {
        PositiveInteger::new(Integer::from(value)).expect("unit-test inputs are positive")
    }

    #[test]
    fn exact_step_and_runner_cover_both_branches_and_normal_stops() {
        assert_eq!(bigint_step(&exact(6)), exact(3));
        assert_eq!(bigint_step(&exact(3)), exact(10));

        let terminal = run_bigint(exact(1), 0);
        assert_eq!(terminal.last, exact(1));
        assert_eq!(terminal.termination, Termination::ReachedOne);

        let limited = run_bigint(exact(8), 1);
        assert_eq!(limited.last, exact(4));
        assert_eq!(limited.observed_peak, exact(8));
        assert_eq!(limited.first_descent_step, Some(1));
        assert_eq!(limited.termination, Termination::StepLimitReached);

        let growth_then_descent = run_bigint(exact(3), 7);
        assert_eq!(growth_then_descent.last, exact(1));
        assert_eq!(growth_then_descent.observed_peak, exact(16));
        assert_eq!(growth_then_descent.first_descent_step, Some(6));
    }
}
