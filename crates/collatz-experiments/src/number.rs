use core::fmt;
use std::str::FromStr;

use collatz_engine::PositiveInteger;
use rug::Integer;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The only number-definition schema accepted by this MVP slice.
pub const NUMBER_DEFINITION_SCHEMA_VERSION: u32 = 1;

/// A reconstructible positive-integer definition plus declared provenance.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumberDefinition {
    pub schema_version: u32,
    pub input_id: String,
    pub name: String,
    pub family: String,
    pub construction: NumberConstruction,
    pub provenance: Provenance,
    pub declared_bit_length: u32,
    pub declared_decimal_digits: u32,
}

/// The supported pure constructors for catalog values.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NumberConstruction {
    Literal { value: String },
    Mersenne { exponent: u32 },
    Fermat { index: u32 },
    Repunit { base: u32, length: u32 },
    AffinePowerOfTwo { coefficient: String, exponent: u32 },
}

impl NumberConstruction {
    /// Returns a stable human-readable reconstruction formula.
    pub fn formula(&self) -> String {
        match self {
            Self::Literal { value } => value.clone(),
            Self::Mersenne { exponent } => format!("2^{exponent} - 1"),
            Self::Fermat { index } => format!("2^(2^{index}) + 1"),
            Self::Repunit { base, length } => {
                format!("({base}^{length} - 1) / ({base} - 1)")
            }
            Self::AffinePowerOfTwo {
                coefficient,
                exponent,
            } => format!("{coefficient} * 2^{exponent} - 1"),
        }
    }

    fn reconstruct(&self) -> Result<Integer, NumberValidationError> {
        match self {
            Self::Literal { value } => parse_canonical_positive_decimal("value", value),
            Self::Mersenne { exponent } => {
                require_positive_parameter("Mersenne exponent", *exponent)?;
                Ok((Integer::from(1) << exponent) - 1)
            }
            Self::Fermat { index } => {
                let shift = 1_u32.checked_shl(*index).ok_or_else(|| {
                    NumberValidationError::ReconstructionLimit {
                        construction: self.formula(),
                        reason: "2^index does not fit the exact-integer shift interface".into(),
                    }
                })?;
                Ok((Integer::from(1) << shift) + 1)
            }
            Self::Repunit { base, length } => {
                if *base < 2 {
                    return Err(NumberValidationError::InvalidDomain {
                        construction: self.formula(),
                        reason: "repunit base must be at least 2".into(),
                    });
                }
                require_positive_parameter("repunit length", *length)?;

                let mut value = Integer::from(0);
                for _ in 0..*length {
                    value *= base;
                    value += 1;
                }
                Ok(value)
            }
            Self::AffinePowerOfTwo {
                coefficient,
                exponent,
            } => {
                let coefficient = parse_canonical_positive_decimal("coefficient", coefficient)?;
                require_positive_parameter("affine exponent", *exponent)?;
                Ok((coefficient << exponent) - 1)
            }
        }
    }
}

/// Whether the exact value is reconstructed locally or imported as data.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueOrigin {
    Generated,
    Imported,
}

/// Source and reconstruction metadata carried into every result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub origin: ValueOrigin,
    pub source: String,
    pub external_id: Option<String>,
    pub retrieval_date: Option<String>,
    pub imported_value_sha256: Option<String>,
    pub reconstruction_note: String,
}

/// A definition whose construction, metadata, and provenance agree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedNumber {
    definition: NumberDefinition,
    value: PositiveInteger,
}

impl ValidatedNumber {
    pub fn validate(definition: NumberDefinition) -> Result<Self, NumberValidationError> {
        if definition.schema_version != NUMBER_DEFINITION_SCHEMA_VERSION {
            return Err(NumberValidationError::UnsupportedSchemaVersion {
                found: definition.schema_version,
            });
        }

        validate_identifier("input_id", &definition.input_id)?;
        validate_nonempty("name", &definition.name)?;
        validate_identifier("family", &definition.family)?;
        validate_nonempty("provenance.source", &definition.provenance.source)?;
        validate_nonempty(
            "provenance.reconstruction_note",
            &definition.provenance.reconstruction_note,
        )?;

        let value = definition.construction.reconstruct()?;
        let canonical_decimal = value.to_string();
        let actual_bit_length = value.significant_bits();
        let actual_decimal_digits = u32::try_from(canonical_decimal.len()).map_err(|_| {
            NumberValidationError::ReconstructionLimit {
                construction: definition.construction.formula(),
                reason: "decimal digit count exceeds u32".into(),
            }
        })?;

        validate_declared_metadata(
            "declared_bit_length",
            definition.declared_bit_length,
            actual_bit_length,
        )?;
        validate_declared_metadata(
            "declared_decimal_digits",
            definition.declared_decimal_digits,
            actual_decimal_digits,
        )?;
        validate_provenance(&definition.provenance, canonical_decimal.as_bytes())?;

        let value =
            PositiveInteger::new(value).map_err(|error| NumberValidationError::InvalidDomain {
                construction: definition.construction.formula(),
                reason: error.to_string(),
            })?;

        Ok(Self { definition, value })
    }

    pub fn definition(&self) -> &NumberDefinition {
        &self.definition
    }

    pub fn value(&self) -> &PositiveInteger {
        &self.value
    }

    pub fn decimal_value(&self) -> String {
        self.value.get().to_string()
    }

    pub fn into_parts(self) -> (NumberDefinition, PositiveInteger) {
        (self.definition, self.value)
    }
}

/// A typed definition failure reported before engine execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NumberValidationError {
    UnsupportedSchemaVersion {
        found: u32,
    },
    EmptyField {
        field: &'static str,
    },
    InvalidIdentifier {
        field: &'static str,
        value: String,
    },
    InvalidDecimal {
        field: &'static str,
        value: String,
    },
    InvalidDomain {
        construction: String,
        reason: String,
    },
    ReconstructionLimit {
        construction: String,
        reason: String,
    },
    MetadataMismatch {
        field: &'static str,
        declared: u32,
        actual: u32,
    },
    ImportedFieldMissing {
        field: &'static str,
    },
    InvalidRetrievalDate {
        value: String,
    },
    InvalidSha256 {
        value: String,
    },
    ImportedSha256Mismatch {
        declared: String,
        actual: String,
    },
    GeneratedValueHasImportedHash,
}

impl NumberValidationError {
    /// Stable experiment-status code for definition failures.
    pub const fn status_code(&self) -> &'static str {
        "invalid_input"
    }
}

impl fmt::Display for NumberValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { found } => {
                write!(formatter, "unsupported number schema version {found}")
            }
            Self::EmptyField { field } => write!(formatter, "{field} must not be empty"),
            Self::InvalidIdentifier { field, value } => write!(
                formatter,
                "{field} must use lowercase ASCII letters, digits, or hyphens: {value}"
            ),
            Self::InvalidDecimal { field, value } => {
                write!(
                    formatter,
                    "{field} is not a canonical positive decimal: {value}"
                )
            }
            Self::InvalidDomain {
                construction,
                reason,
            } => write!(formatter, "invalid construction {construction}: {reason}"),
            Self::ReconstructionLimit {
                construction,
                reason,
            } => write!(
                formatter,
                "cannot reconstruct {construction} on this platform: {reason}"
            ),
            Self::MetadataMismatch {
                field,
                declared,
                actual,
            } => write!(
                formatter,
                "{field} mismatch: declared {declared}, reconstructed {actual}"
            ),
            Self::ImportedFieldMissing { field } => {
                write!(formatter, "imported value requires {field}")
            }
            Self::InvalidRetrievalDate { value } => {
                write!(formatter, "retrieval_date must be YYYY-MM-DD: {value}")
            }
            Self::InvalidSha256 { value } => {
                write!(
                    formatter,
                    "imported_value_sha256 must be 64 lowercase hex digits: {value}"
                )
            }
            Self::ImportedSha256Mismatch { declared, actual } => write!(
                formatter,
                "imported SHA-256 mismatch: declared {declared}, reconstructed {actual}"
            ),
            Self::GeneratedValueHasImportedHash => formatter
                .write_str("locally generated value must not declare imported_value_sha256"),
        }
    }
}

impl std::error::Error for NumberValidationError {}

fn parse_canonical_positive_decimal(
    field: &'static str,
    text: &str,
) -> Result<Integer, NumberValidationError> {
    let value = Integer::from_str(text).map_err(|_| NumberValidationError::InvalidDecimal {
        field,
        value: text.into(),
    })?;
    if value <= 0 || value.to_string() != text {
        return Err(NumberValidationError::InvalidDecimal {
            field,
            value: text.into(),
        });
    }
    Ok(value)
}

fn require_positive_parameter(name: &'static str, value: u32) -> Result<(), NumberValidationError> {
    if value == 0 {
        Err(NumberValidationError::InvalidDomain {
            construction: name.into(),
            reason: "must be at least 1".into(),
        })
    } else {
        Ok(())
    }
}

fn validate_identifier(field: &'static str, value: &str) -> Result<(), NumberValidationError> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        Err(NumberValidationError::InvalidIdentifier {
            field,
            value: value.into(),
        })
    } else {
        Ok(())
    }
}

fn validate_nonempty(field: &'static str, value: &str) -> Result<(), NumberValidationError> {
    if value.trim().is_empty() {
        Err(NumberValidationError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn validate_declared_metadata(
    field: &'static str,
    declared: u32,
    actual: u32,
) -> Result<(), NumberValidationError> {
    if declared == actual {
        Ok(())
    } else {
        Err(NumberValidationError::MetadataMismatch {
            field,
            declared,
            actual,
        })
    }
}

fn validate_provenance(
    provenance: &Provenance,
    canonical_decimal: &[u8],
) -> Result<(), NumberValidationError> {
    match provenance.origin {
        ValueOrigin::Generated => {
            if provenance.imported_value_sha256.is_some() {
                return Err(NumberValidationError::GeneratedValueHasImportedHash);
            }
        }
        ValueOrigin::Imported => {
            require_present("provenance.external_id", provenance.external_id.as_deref())?;
            let retrieval_date = require_present(
                "provenance.retrieval_date",
                provenance.retrieval_date.as_deref(),
            )?;
            validate_date(retrieval_date)?;
            let declared_hash = require_present(
                "provenance.imported_value_sha256",
                provenance.imported_value_sha256.as_deref(),
            )?;
            validate_lower_hex_sha256(declared_hash)?;

            let actual_hash = hex_sha256(canonical_decimal);
            if declared_hash != actual_hash {
                return Err(NumberValidationError::ImportedSha256Mismatch {
                    declared: declared_hash.into(),
                    actual: actual_hash,
                });
            }
        }
    }
    Ok(())
}

fn require_present<'a>(
    field: &'static str,
    value: Option<&'a str>,
) -> Result<&'a str, NumberValidationError> {
    value
        .filter(|value| !value.trim().is_empty())
        .ok_or(NumberValidationError::ImportedFieldMissing { field })
}

fn validate_date(value: &str) -> Result<(), NumberValidationError> {
    let bytes = value.as_bytes();
    let shape_is_valid = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit());

    if !shape_is_valid {
        return Err(NumberValidationError::InvalidRetrievalDate {
            value: value.into(),
        });
    }

    let month = value[5..7].parse::<u8>().unwrap_or(0);
    let day = value[8..10].parse::<u8>().unwrap_or(0);
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(NumberValidationError::InvalidRetrievalDate {
            value: value.into(),
        });
    }
    Ok(())
}

fn validate_lower_hex_sha256(value: &str) -> Result<(), NumberValidationError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Err(NumberValidationError::InvalidSha256 {
            value: value.into(),
        })
    } else {
        Ok(())
    }
}

pub(crate) fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use core::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}
