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
