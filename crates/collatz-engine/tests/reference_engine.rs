use collatz_engine::{
    MersenneError, PositiveU128, PositiveU128Error, Termination, mersenne, run, step,
};

fn positive(value: u128) -> PositiveU128 {
    PositiveU128::new(value).expect("test inputs are positive literals")
}

#[test]
fn zero_is_rejected_by_the_positive_domain_type() {
    assert_eq!(PositiveU128::new(0), Err(PositiveU128Error::Zero));
    assert_eq!(PositiveU128::try_from(0), Err(PositiveU128Error::Zero));
}

#[test]
fn positive_domain_value_round_trips() {
    let value = positive(42);

    assert_eq!(value.get(), 42);
    assert_eq!(u128::from(value), 42);
}

#[test]
fn one_even_classical_step_is_division_by_two() {
    assert_eq!(step(positive(6)), Ok(positive(3)));
}

#[test]
fn one_odd_classical_step_is_three_n_plus_one() {
    assert_eq!(step(positive(3)), Ok(positive(10)));
}

#[test]
fn fixed_known_values_match_the_independent_oracles() {
    let cases = [(1, 0, 1), (2, 1, 2), (3, 7, 16), (27, 111, 9_232)];

    for (start, expected_steps, expected_peak) in cases {
        let summary = run(positive(start), expected_steps).expect("known vector reaches one");

        assert_eq!(summary.start, positive(start));
        assert_eq!(summary.last, positive(1));
        assert_eq!(summary.completed_classical_steps, expected_steps);
        assert_eq!(summary.classical_step_limit, expected_steps);
        assert_eq!(summary.observed_peak, positive(expected_peak));
        assert_eq!(summary.termination, Termination::ReachedOne);
    }
}

#[test]
fn terminal_one_has_priority_over_a_zero_limit() {
    let summary = run(positive(1), 0).expect("one is already terminal");

    assert_eq!(summary.start, positive(1));
    assert_eq!(summary.last, positive(1));
    assert_eq!(summary.completed_classical_steps, 0);
    assert_eq!(summary.classical_step_limit, 0);
    assert_eq!(summary.observed_peak, positive(1));
    assert_eq!(summary.first_descent_step, None);
    assert_eq!(summary.termination, Termination::ReachedOne);
}

#[test]
fn nonterminal_zero_limit_is_a_zero_transition_prefix() {
    let summary = run(positive(2), 0).expect("limit exhaustion is a normal termination");

    assert_eq!(summary.start, positive(2));
    assert_eq!(summary.last, positive(2));
    assert_eq!(summary.completed_classical_steps, 0);
    assert_eq!(summary.classical_step_limit, 0);
    assert_eq!(summary.observed_peak, positive(2));
    assert_eq!(summary.first_descent_step, None);
    assert_eq!(summary.termination, Termination::StepLimitReached);
}

#[test]
fn reaching_one_on_the_last_permitted_step_is_complete() {
    let summary = run(positive(2), 1).expect("the last permitted transition reaches one");

    assert_eq!(summary.last, positive(1));
    assert_eq!(summary.completed_classical_steps, 1);
    assert_eq!(summary.termination, Termination::ReachedOne);
}

#[test]
fn the_initial_value_participates_in_peak_and_descent_accounting() {
    let summary = run(positive(8), 1).expect("one even step is representable");

    assert_eq!(summary.last, positive(4));
    assert_eq!(summary.observed_peak, positive(8));
    assert_eq!(summary.first_descent_step, Some(1));
    assert_eq!(summary.termination, Termination::StepLimitReached);
}

#[test]
fn a_limit_check_prevents_an_unpermitted_overflowing_step() {
    let overflow_risk = u128::MAX / 3;
    let summary = run(positive(overflow_risk), 0).expect("zero limit attempts no transition");

    assert_eq!(summary.last, positive(overflow_risk));
    assert_eq!(summary.completed_classical_steps, 0);
    assert_eq!(summary.termination, Termination::StepLimitReached);
}

#[test]
fn odd_addition_overflow_is_reported_at_the_current_value() {
    let overflow_risk = positive(u128::MAX / 3);
    let error = step(overflow_risk).expect_err("3n + 1 is not representable");

    assert_eq!(error.current, overflow_risk);
}

#[test]
fn odd_multiplication_overflow_is_reported_at_the_current_value() {
    let overflow_risk = positive(u128::MAX);
    let error = step(overflow_risk).expect_err("3n is not representable");

    assert_eq!(error.current, overflow_risk);
}

#[test]
fn runner_overflow_does_not_count_or_invent_a_transition() {
    let overflow_risk = positive(u128::MAX / 3);
    let error = run(overflow_risk, 1).expect_err("the first transition overflows");

    assert_eq!(error.overflow.current, overflow_risk);
    assert_eq!(error.progress.start, overflow_risk);
    assert_eq!(error.progress.last, overflow_risk);
    assert_eq!(error.progress.completed_classical_steps, 0);
    assert_eq!(error.progress.classical_step_limit, 1);
    assert_eq!(error.progress.observed_peak, overflow_risk);
    assert_eq!(error.progress.first_descent_step, None);
}

#[test]
fn runner_preserves_progress_before_a_later_overflow() {
    let overflow_risk = u128::MAX / 3;
    let start = positive(overflow_risk * 2);
    let error = run(start, 2).expect_err("the second transition overflows");

    assert_eq!(error.overflow.current, positive(overflow_risk));
    assert_eq!(error.progress.start, start);
    assert_eq!(error.progress.last, positive(overflow_risk));
    assert_eq!(error.progress.completed_classical_steps, 1);
    assert_eq!(error.progress.classical_step_limit, 2);
    assert_eq!(error.progress.observed_peak, start);
    assert_eq!(error.progress.first_descent_step, Some(1));
}

#[test]
fn largest_safe_odd_transition_is_represented_exactly() {
    let largest_safe_odd = u128::MAX / 3 - 2;

    assert_eq!(
        step(positive(largest_safe_odd)),
        Ok(positive(u128::MAX - 5))
    );
}

#[test]
fn small_mersenne_generator_matches_the_reviewed_vector() {
    assert_eq!(mersenne(5), Ok(positive(31)));
}

#[test]
fn mersenne_generator_enforces_its_domain_and_u128_boundary() {
    assert_eq!(mersenne(0), Err(MersenneError::ZeroExponent));
    assert_eq!(mersenne(1), Ok(positive(1)));
    assert_eq!(mersenne(128), Ok(positive(u128::MAX)));
    assert_eq!(
        mersenne(129),
        Err(MersenneError::ExponentTooLarge { exponent: 129 })
    );
}
