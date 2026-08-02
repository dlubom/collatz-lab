use collatz_engine::{HybridValue, PositiveInteger, PositiveU128, run, run_bigint, run_hybrid};
use proptest::prelude::*;
use rug::Integer;

fn bounded(value: u128) -> PositiveU128 {
    PositiveU128::new(value).expect("test inputs are positive")
}

fn exact(value: u128) -> PositiveInteger {
    PositiveInteger::new(Integer::from(value)).expect("test inputs are positive")
}

proptest! {
    #[test]
    fn common_domain_runs_agree_on_all_public_metrics(
        start in 1_u128..=1_000_000,
        limit in 0_u64..=512,
    ) {
        let reference = run(bounded(start), limit)
            .expect("generated small prefixes remain representable as u128");
        let bigint = run_bigint(exact(start), limit);
        let hybrid = run_hybrid(bounded(start), limit);

        prop_assert_eq!(bigint.start.get(), &Integer::from(reference.start.get()));
        prop_assert_eq!(bigint.last.get(), &Integer::from(reference.last.get()));
        prop_assert_eq!(
            bigint.completed_classical_steps,
            reference.completed_classical_steps
        );
        prop_assert_eq!(bigint.classical_step_limit, reference.classical_step_limit);
        prop_assert_eq!(
            bigint.observed_peak.get(),
            &Integer::from(reference.observed_peak.get())
        );
        prop_assert_eq!(bigint.first_descent_step, reference.first_descent_step);
        prop_assert_eq!(bigint.termination, reference.termination);

        prop_assert_eq!(hybrid.start, reference.start);
        prop_assert_eq!(hybrid.last.as_u128(), Some(reference.last));
        prop_assert_eq!(
            hybrid.completed_classical_steps,
            reference.completed_classical_steps
        );
        prop_assert_eq!(hybrid.classical_step_limit, reference.classical_step_limit);
        prop_assert_eq!(hybrid.observed_peak.as_u128(), Some(reference.observed_peak));
        prop_assert_eq!(hybrid.first_descent_step, reference.first_descent_step);
        prop_assert_eq!(hybrid.promotion_count, 0);
        prop_assert_eq!(hybrid.termination, reference.termination);
    }
}

#[test]
fn maximum_u128_promotes_before_the_exact_odd_transition() {
    let start = bounded(u128::MAX);
    let expected = Integer::from(u128::MAX) * 3 + 1;

    let reference_error = run(start, 1).expect_err("bounded execution must report overflow");
    assert_eq!(reference_error.overflow.current, start);
    assert_eq!(reference_error.progress.last, start);
    assert_eq!(reference_error.progress.completed_classical_steps, 0);

    let bigint = run_bigint(PositiveInteger::from(start), 1);
    assert_eq!(bigint.last.get(), &expected);
    assert_eq!(bigint.completed_classical_steps, 1);

    let hybrid = run_hybrid(start, 1);
    assert_eq!(hybrid.last.as_u128(), None);
    assert_eq!(
        hybrid.last.as_bigint().map(PositiveInteger::get),
        Some(&expected)
    );
    assert_eq!(hybrid.last.to_integer(), expected);
    assert_eq!(hybrid.completed_classical_steps, 1);
    assert_eq!(
        hybrid.observed_peak.to_integer(),
        bigint.observed_peak.into_inner()
    );
    assert_eq!(hybrid.first_descent_step, None);
    assert_eq!(hybrid.promotion_count, 1);
    assert_eq!(hybrid.termination, bigint.termination);
}

#[test]
fn promotion_after_a_bounded_prefix_neither_repeats_nor_omits_a_step() {
    let overflow_risk = u128::MAX / 3;
    let start = bounded(overflow_risk * 2);

    let bigint = run_bigint(PositiveInteger::from(start), 2);
    let hybrid = run_hybrid(start, 2);

    assert_eq!(hybrid.completed_classical_steps, 2);
    assert_eq!(hybrid.last.to_integer(), bigint.last.into_inner());
    assert_eq!(
        hybrid.observed_peak.to_integer(),
        bigint.observed_peak.into_inner()
    );
    assert_eq!(hybrid.first_descent_step, Some(1));
    assert_eq!(hybrid.promotion_count, 1);
    assert_eq!(hybrid.termination, bigint.termination);
}

#[test]
fn a_limit_can_stop_before_promotion_and_a_promoted_run_never_demotes() {
    let start = bounded(u128::MAX);
    let stopped = run_hybrid(start, 0);
    assert_eq!(stopped.last, HybridValue::U128(start));
    assert_eq!(stopped.promotion_count, 0);

    let promoted = run_hybrid(start, 4);
    assert!(matches!(promoted.last, HybridValue::BigInt(_)));
    assert!(matches!(promoted.observed_peak, HybridValue::BigInt(_)));
    assert_eq!(promoted.promotion_count, 1);
}

#[test]
fn every_observed_post_promotion_prefix_matches_bigint_from_start() {
    let start = bounded(u128::MAX);

    for limit in 1..=32 {
        let bigint = run_bigint(PositiveInteger::from(start), limit);
        let hybrid = run_hybrid(start, limit);

        assert!(matches!(hybrid.last, HybridValue::BigInt(_)));
        assert!(matches!(hybrid.observed_peak, HybridValue::BigInt(_)));
        assert_eq!(hybrid.last.to_integer(), bigint.last.into_inner());
        assert_eq!(
            hybrid.observed_peak.to_integer(),
            bigint.observed_peak.into_inner()
        );
        assert_eq!(
            hybrid.completed_classical_steps,
            bigint.completed_classical_steps
        );
        assert_eq!(hybrid.classical_step_limit, bigint.classical_step_limit);
        assert_eq!(hybrid.first_descent_step, bigint.first_descent_step);
        assert_eq!(hybrid.promotion_count, 1);
        assert_eq!(hybrid.termination, bigint.termination);
    }
}
