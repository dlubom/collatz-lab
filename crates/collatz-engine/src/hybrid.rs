use crate::{
    HybridRunSummary, HybridValue, PositiveInteger, PositiveU128, Termination, bigint_step,
};

/// Evaluates a finite trajectory that promotes from `u128` before overflow.
///
/// Promotion converts the current value before performing the exact odd
/// transition. The runner never demotes and records at most one promotion.
pub fn run_hybrid(start: PositiveU128, classical_step_limit: u64) -> HybridRunSummary {
    let mut current = HybridValue::U128(start);
    let mut completed_classical_steps = 0;
    let mut observed_peak = HybridValue::U128(start);
    let mut first_descent_step = None;
    let mut promotion_count = 0;

    loop {
        if is_one(&current) {
            return summary(
                start,
                classical_step_limit,
                HybridProgress {
                    last: current,
                    completed_classical_steps,
                    observed_peak,
                    first_descent_step,
                    promotion_count,
                },
                Termination::ReachedOne,
            );
        }

        if completed_classical_steps == classical_step_limit {
            return summary(
                start,
                classical_step_limit,
                HybridProgress {
                    last: current,
                    completed_classical_steps,
                    observed_peak,
                    first_descent_step,
                    promotion_count,
                },
                Termination::StepLimitReached,
            );
        }

        let (next, promoted) = next_value(&current);
        if promoted {
            promotion_count = 1;
        }

        completed_classical_steps += 1;
        current = next;
        observed_peak = updated_peak(observed_peak, &current);

        if first_descent_step.is_none() && is_below_bounded_start(&current, start) {
            first_descent_step = Some(completed_classical_steps);
        }
    }
}

fn next_value(current: &HybridValue) -> (HybridValue, bool) {
    match current {
        HybridValue::U128(current) => {
            let value = current.get();

            if value.is_multiple_of(2) {
                (HybridValue::U128(PositiveU128(value / 2)), false)
            } else {
                match value
                    .checked_mul(3)
                    .and_then(|product| product.checked_add(1))
                {
                    Some(next) => (HybridValue::U128(PositiveU128(next)), false),
                    None => {
                        let promoted_current = PositiveInteger::from(*current);
                        let next = bigint_step(&promoted_current);
                        (HybridValue::BigInt(next), true)
                    }
                }
            }
        }
        HybridValue::BigInt(current) => (HybridValue::BigInt(bigint_step(current)), false),
    }
}

fn is_one(value: &HybridValue) -> bool {
    match value {
        HybridValue::U128(value) => value.get() == 1,
        HybridValue::BigInt(value) => value.get() == &1,
    }
}

fn is_below_bounded_start(value: &HybridValue, start: PositiveU128) -> bool {
    match value {
        HybridValue::U128(value) => *value < start,
        HybridValue::BigInt(value) => value.get() < &start.get(),
    }
}

fn updated_peak(peak: HybridValue, current: &HybridValue) -> HybridValue {
    match (peak, current) {
        (HybridValue::U128(peak), HybridValue::U128(current)) => {
            HybridValue::U128(peak.max(*current))
        }
        (HybridValue::U128(peak), HybridValue::BigInt(current)) => {
            let peak = PositiveInteger::from(peak);
            HybridValue::BigInt(if current > &peak {
                current.clone()
            } else {
                peak
            })
        }
        (HybridValue::BigInt(peak), HybridValue::BigInt(current)) => {
            HybridValue::BigInt(if current > &peak {
                current.clone()
            } else {
                peak
            })
        }
        (HybridValue::BigInt(peak), HybridValue::U128(current)) => {
            let current = PositiveInteger::from(*current);
            HybridValue::BigInt(if current > peak { current } else { peak })
        }
    }
}

struct HybridProgress {
    last: HybridValue,
    completed_classical_steps: u64,
    observed_peak: HybridValue,
    first_descent_step: Option<u64>,
    promotion_count: u8,
}

fn summary(
    start: PositiveU128,
    classical_step_limit: u64,
    progress: HybridProgress,
    termination: Termination,
) -> HybridRunSummary {
    HybridRunSummary {
        start,
        last: progress.last,
        completed_classical_steps: progress.completed_classical_steps,
        classical_step_limit,
        observed_peak: progress.observed_peak,
        first_descent_step: progress.first_descent_step,
        promotion_count: progress.promotion_count,
        termination,
    }
}

#[cfg(test)]
mod tests {
    use rug::Integer;

    use super::*;

    fn positive(value: u128) -> PositiveU128 {
        PositiveU128::new(value).expect("unit-test inputs are positive")
    }

    #[test]
    fn hybrid_runner_covers_bounded_terminal_limit_and_metrics() {
        let terminal = run_hybrid(positive(1), 0);
        assert_eq!(terminal.last, HybridValue::U128(positive(1)));
        assert_eq!(terminal.termination, Termination::ReachedOne);

        let limited = run_hybrid(positive(2), 0);
        assert_eq!(limited.last, HybridValue::U128(positive(2)));
        assert_eq!(limited.termination, Termination::StepLimitReached);

        let complete = run_hybrid(positive(3), 7);
        assert_eq!(complete.last, HybridValue::U128(positive(1)));
        assert_eq!(complete.observed_peak, HybridValue::U128(positive(16)));
        assert_eq!(complete.first_descent_step, Some(6));
        assert_eq!(complete.promotion_count, 0);
    }

    #[test]
    fn hybrid_runner_promotes_current_value_once_and_never_demotes() {
        let summary = run_hybrid(positive(u128::MAX), 4);
        let expected_peak: Integer = (Integer::from(9) << 127) - 2;

        assert_eq!(summary.completed_classical_steps, 4);
        assert_eq!(summary.promotion_count, 1);
        assert_eq!(summary.first_descent_step, None);
        assert_eq!(summary.observed_peak.to_integer(), expected_peak);
        assert!(summary.last.as_u128().is_none());
        assert!(summary.last.as_bigint().is_some());

        let exact_peak = HybridValue::BigInt(PositiveInteger::from(positive(10)));
        let bounded_current = HybridValue::U128(positive(12));
        assert_eq!(
            updated_peak(exact_peak, &bounded_current).to_integer(),
            Integer::from(12)
        );
    }
}
