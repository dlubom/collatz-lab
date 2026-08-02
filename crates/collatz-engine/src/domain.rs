use core::fmt;

use rug::Integer;

/// A positive integer representable by the bounded reference engine.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PositiveU128(pub(crate) u128);

impl PositiveU128 {
    /// Validates and constructs a positive bounded-engine value.
    pub const fn new(value: u128) -> Result<Self, PositiveU128Error> {
        if value == 0 {
            Err(PositiveU128Error::Zero)
        } else {
            Ok(Self(value))
        }
    }

    /// Returns the represented integer.
    pub const fn get(self) -> u128 {
        self.0
    }
}

impl TryFrom<u128> for PositiveU128 {
    type Error = PositiveU128Error;

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<PositiveU128> for u128 {
    fn from(value: PositiveU128) -> Self {
        value.get()
    }
}

/// Failure to construct a value in the positive input domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PositiveU128Error {
    /// Zero is outside the mathematical input domain.
    Zero,
}

impl fmt::Display for PositiveU128Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zero => formatter.write_str("Collatz input must be positive"),
        }
    }
}

impl std::error::Error for PositiveU128Error {}

/// A positive integer represented exactly with arbitrary precision.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PositiveInteger(pub(crate) Integer);

impl PositiveInteger {
    /// Validates and constructs a positive arbitrary-precision value.
    pub fn new(value: Integer) -> Result<Self, PositiveIntegerError> {
        if value == 0 {
            Err(PositiveIntegerError::Zero)
        } else if value < 0 {
            Err(PositiveIntegerError::Negative)
        } else {
            Ok(Self(value))
        }
    }

    /// Borrows the represented integer without narrowing it.
    pub fn get(&self) -> &Integer {
        &self.0
    }

    /// Returns the represented integer without narrowing it.
    pub fn into_inner(self) -> Integer {
        self.0
    }
}

impl TryFrom<Integer> for PositiveInteger {
    type Error = PositiveIntegerError;

    fn try_from(value: Integer) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<PositiveU128> for PositiveInteger {
    fn from(value: PositiveU128) -> Self {
        Self(Integer::from(value.get()))
    }
}

impl From<PositiveInteger> for Integer {
    fn from(value: PositiveInteger) -> Self {
        value.into_inner()
    }
}

/// Failure to construct an arbitrary-precision value in the positive domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PositiveIntegerError {
    /// Zero is outside the mathematical input domain.
    Zero,
    /// Negative integers are outside the mathematical input domain.
    Negative,
}

impl fmt::Display for PositiveIntegerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zero => formatter.write_str("Collatz input must be positive, not zero"),
            Self::Negative => formatter.write_str("Collatz input must be positive, not negative"),
        }
    }
}

impl std::error::Error for PositiveIntegerError {}

/// A normal finite-run termination classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Termination {
    /// The runner observed terminal value `1`.
    ReachedOne,
    /// The runner used the complete classical-transition budget without
    /// observing `1`.
    StepLimitReached,
}

/// Metrics for a normally terminated bounded run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RunSummary {
    /// Validated starting value.
    pub start: PositiveU128,
    /// Last successfully represented classical value.
    pub last: PositiveU128,
    /// Number of successfully completed classical transitions.
    pub completed_classical_steps: u64,
    /// Configured maximum number of classical transitions.
    pub classical_step_limit: u64,
    /// Maximum over the represented classical prefix, including `start`.
    pub observed_peak: PositiveU128,
    /// First completed-step index whose value is below `start`, when observed.
    pub first_descent_step: Option<u64>,
    /// Why this finite observation stopped normally.
    pub termination: Termination,
}

/// Metrics for a normally terminated arbitrary-precision run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BigIntRunSummary {
    /// Validated starting value.
    pub start: PositiveInteger,
    /// Last successfully represented classical value.
    pub last: PositiveInteger,
    /// Number of successfully completed classical transitions.
    pub completed_classical_steps: u64,
    /// Configured maximum number of classical transitions.
    pub classical_step_limit: u64,
    /// Maximum over the represented classical prefix, including `start`.
    pub observed_peak: PositiveInteger,
    /// First completed-step index whose value is below `start`, when observed.
    pub first_descent_step: Option<u64>,
    /// Why this finite observation stopped normally.
    pub termination: Termination,
}

/// The active numeric representation of a hybrid run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HybridValue {
    /// A value still represented by the bounded `u128` engine state.
    U128(PositiveU128),
    /// A value represented exactly after one-way promotion.
    BigInt(PositiveInteger),
}

impl HybridValue {
    /// Returns the bounded value when the hybrid runner has not promoted.
    pub const fn as_u128(&self) -> Option<PositiveU128> {
        match self {
            Self::U128(value) => Some(*value),
            Self::BigInt(_) => None,
        }
    }

    /// Borrows the exact value when the hybrid runner has promoted.
    pub fn as_bigint(&self) -> Option<&PositiveInteger> {
        match self {
            Self::U128(_) => None,
            Self::BigInt(value) => Some(value),
        }
    }

    /// Clones the represented mathematical integer without narrowing it.
    pub fn to_integer(&self) -> Integer {
        match self {
            Self::U128(value) => Integer::from(value.get()),
            Self::BigInt(value) => value.get().clone(),
        }
    }
}

/// Metrics for a normally terminated hybrid run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HybridRunSummary {
    /// Validated `u128` starting value.
    pub start: PositiveU128,
    /// Last successfully represented classical value and its active representation.
    pub last: HybridValue,
    /// Number of successfully completed classical transitions.
    pub completed_classical_steps: u64,
    /// Configured maximum number of classical transitions.
    pub classical_step_limit: u64,
    /// Maximum over the represented classical prefix, including `start`.
    pub observed_peak: HybridValue,
    /// First completed-step index whose value is below `start`, when observed.
    pub first_descent_step: Option<u64>,
    /// Number of one-way `u128`-to-BigInt promotions; always zero or one.
    pub promotion_count: u8,
    /// Why this finite observation stopped normally.
    pub termination: Termination,
}

/// Metrics available when a bounded run cannot represent its next transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RunProgress {
    /// Validated starting value.
    pub start: PositiveU128,
    /// Last successfully represented classical value.
    pub last: PositiveU128,
    /// Number of successfully completed classical transitions.
    pub completed_classical_steps: u64,
    /// Configured maximum number of classical transitions.
    pub classical_step_limit: u64,
    /// Maximum over the represented classical prefix, including `start`.
    pub observed_peak: PositiveU128,
    /// First completed-step index whose value is below `start`, when observed.
    pub first_descent_step: Option<u64>,
}

/// A checked odd transition whose result is not representable as `u128`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArithmeticOverflow {
    /// Current value at which the next classical transition failed.
    pub current: PositiveU128,
}

impl fmt::Display for ArithmeticOverflow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "3n + 1 is not representable as u128 at n={}",
            self.current.get()
        )
    }
}

impl std::error::Error for ArithmeticOverflow {}

/// Typed failure of a finite bounded run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RunError {
    /// The unrepresentable transition.
    pub overflow: ArithmeticOverflow,
    /// Complete metrics through the last represented value.
    pub progress: RunProgress,
}

impl fmt::Display for RunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} after {} completed classical steps",
            self.overflow, self.progress.completed_classical_steps
        )
    }
}

impl std::error::Error for RunError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.overflow)
    }
}
