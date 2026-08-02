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
