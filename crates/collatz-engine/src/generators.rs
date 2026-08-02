use core::fmt;

use crate::PositiveU128;

/// Generates `2^exponent - 1` in the bounded positive domain.
pub fn mersenne(exponent: u32) -> Result<PositiveU128, MersenneError> {
    match exponent {
        0 => Err(MersenneError::ZeroExponent),
        1..=127 => Ok(PositiveU128((1_u128 << exponent) - 1)),
        128 => Ok(PositiveU128(u128::MAX)),
        _ => Err(MersenneError::ExponentTooLarge { exponent }),
    }
}

/// Failure to construct a bounded positive Mersenne value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MersenneError {
    /// The mathematical generator requires `exponent >= 1`.
    ZeroExponent,
    /// The exact value exceeds the `u128` representation.
    ExponentTooLarge {
        /// Requested exponent.
        exponent: u32,
    },
}

impl fmt::Display for MersenneError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroExponent => formatter.write_str("Mersenne exponent must be at least 1"),
            Self::ExponentTooLarge { exponent } => write!(
                formatter,
                "Mersenne exponent {exponent} exceeds the u128 boundary of 128"
            ),
        }
    }
}

impl std::error::Error for MersenneError {}
