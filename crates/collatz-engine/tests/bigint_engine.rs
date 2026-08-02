use collatz_engine::{
    PositiveInteger, PositiveIntegerError, PositiveU128, Termination, bigint_step, run_bigint,
};
use rug::Integer;

fn exact(value: u128) -> PositiveInteger {
    PositiveInteger::new(Integer::from(value)).expect("test inputs are positive literals")
}

fn bounded(value: u128) -> PositiveU128 {
    PositiveU128::new(value).expect("test inputs are positive literals")
}

#[test]
fn arbitrary_precision_domain_rejects_nonpositive_values() {
    assert_eq!(
        PositiveInteger::new(Integer::from(0)),
        Err(PositiveIntegerError::Zero)
    );
    assert_eq!(
        PositiveInteger::new(Integer::from(-1)),
        Err(PositiveIntegerError::Negative)
    );
    assert_eq!(
        PositiveIntegerError::Zero.to_string(),
        "Collatz input must be positive, not zero"
    );
    assert_eq!(
        PositiveIntegerError::Negative.to_string(),
        "Collatz input must be positive, not negative"
    );
}

#[test]
fn arbitrary_precision_domain_converts_without_narrowing() {
    let from_bounded = PositiveInteger::from(bounded(u128::MAX));
    assert_eq!(from_bounded.get(), &Integer::from(u128::MAX));

    let beyond_u128: Integer = (Integer::from(1) << 192) + 7;
    let positive = PositiveInteger::try_from(beyond_u128.clone())
        .expect("constructed arbitrary-precision value is positive");
    assert_eq!(positive.get(), &beyond_u128);
    assert_eq!(Integer::from(positive), beyond_u128);
}

#[test]
fn exact_classical_step_covers_both_branches() {
    assert_eq!(bigint_step(&exact(6)), exact(3));
    assert_eq!(bigint_step(&exact(3)), exact(10));
    assert_eq!(bigint_step(&exact(1)), exact(4));
}

#[test]
fn fixed_known_values_match_the_independent_oracles() {
    let cases = [(1, 0, 1), (2, 1, 2), (3, 7, 16), (27, 111, 9_232)];

    for (start, expected_steps, expected_peak) in cases {
        let summary = run_bigint(exact(start), expected_steps);

        assert_eq!(summary.start, exact(start));
        assert_eq!(summary.last, exact(1));
        assert_eq!(summary.completed_classical_steps, expected_steps);
        assert_eq!(summary.classical_step_limit, expected_steps);
        assert_eq!(summary.observed_peak, exact(expected_peak));
        assert_eq!(summary.termination, Termination::ReachedOne);
    }
}

#[test]
fn exact_runner_preserves_terminal_limit_peak_and_descent_semantics() {
    let terminal = run_bigint(exact(1), 0);
    assert_eq!(terminal.completed_classical_steps, 0);
    assert_eq!(terminal.observed_peak, exact(1));
    assert_eq!(terminal.first_descent_step, None);
    assert_eq!(terminal.termination, Termination::ReachedOne);

    let limited = run_bigint(exact(8), 1);
    assert_eq!(limited.last, exact(4));
    assert_eq!(limited.completed_classical_steps, 1);
    assert_eq!(limited.observed_peak, exact(8));
    assert_eq!(limited.first_descent_step, Some(1));
    assert_eq!(limited.termination, Termination::StepLimitReached);
}

#[test]
fn medium_mersenne_step_remains_exact_beyond_u128() {
    let input: Integer = (Integer::from(1) << 256) - 1;
    let expected: Integer = (Integer::from(3) << 256) - 2;
    let input = PositiveInteger::new(input).expect("Mersenne value is positive");

    let next = bigint_step(&input);

    assert_eq!(next.get(), &expected);
    assert!(next.get().to_u128().is_none());
}
