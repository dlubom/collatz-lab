use crate::{ArithmeticOverflow, PositiveU128, RunError, RunProgress, RunSummary, Termination};

/// Applies one standard, unaccelerated Collatz transition.
///
/// The odd branch uses checked multiplication and addition. An unrepresentable
/// result is reported at `current`; no wrapped or saturated value is produced.
pub fn step(current: PositiveU128) -> Result<PositiveU128, ArithmeticOverflow> {
    let value = current.get();

    if value.is_multiple_of(2) {
        Ok(PositiveU128(value / 2))
    } else {
        value
            .checked_mul(3)
            .and_then(|product| product.checked_add(1))
            .map(PositiveU128)
            .ok_or(ArithmeticOverflow { current })
    }
}

/// Evaluates a finite classical trajectory with terminal-before-limit ordering.
///
/// A successful result is either a complete observation through the first `1`
/// or an exactly limited prefix. Arithmetic overflow is a separate error that
/// retains metrics through the last represented value and does not count the
/// failed transition.
pub fn run(start: PositiveU128, classical_step_limit: u64) -> Result<RunSummary, RunError> {
    let mut current = start;
    let mut completed_classical_steps = 0;
    let mut observed_peak = start;
    let mut first_descent_step = None;

    loop {
        if current.get() == 1 {
            return Ok(summary(
                start,
                current,
                completed_classical_steps,
                observed_peak,
                first_descent_step,
                Termination::ReachedOne,
            ));
        }

        if completed_classical_steps == classical_step_limit {
            return Ok(summary(
                start,
                current,
                completed_classical_steps,
                observed_peak,
                first_descent_step,
                Termination::StepLimitReached,
            ));
        }

        let next = step(current).map_err(|overflow| RunError {
            overflow,
            progress: RunProgress {
                start,
                last: current,
                completed_classical_steps,
                observed_peak,
                first_descent_step,
            },
        })?;

        completed_classical_steps += 1;
        current = next;

        observed_peak = observed_peak.max(current);
        if is_unobserved_strict_descent(first_descent_step, start, current) {
            first_descent_step = Some(completed_classical_steps);
        }
    }
}

fn is_unobserved_strict_descent(
    first_descent_step: Option<u64>,
    start: PositiveU128,
    current: PositiveU128,
) -> bool {
    first_descent_step.is_none() && current < start
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_descent_requires_a_strict_drop_and_is_recorded_once() {
        let start = PositiveU128(8);

        assert!(!is_unobserved_strict_descent(None, start, start));
        assert!(is_unobserved_strict_descent(None, start, PositiveU128(7)));
        assert!(!is_unobserved_strict_descent(
            Some(1),
            start,
            PositiveU128(4)
        ));
    }
}

fn summary(
    start: PositiveU128,
    last: PositiveU128,
    completed_classical_steps: u64,
    observed_peak: PositiveU128,
    first_descent_step: Option<u64>,
    termination: Termination,
) -> RunSummary {
    RunSummary {
        start,
        last,
        completed_classical_steps,
        observed_peak,
        first_descent_step,
        termination,
    }
}
