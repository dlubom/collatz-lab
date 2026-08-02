use core::fmt;

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
    /// Maximum over the represented classical prefix, including `start`.
    pub observed_peak: PositiveU128,
    /// First completed-step index whose value is below `start`, when observed.
    pub first_descent_step: Option<u64>,
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
