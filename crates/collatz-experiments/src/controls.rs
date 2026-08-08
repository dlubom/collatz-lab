use core::fmt;
use std::collections::HashSet;

use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::{Rng as _, SeedableRng as _};
use rug::Integer;
use rug::integer::Order;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    NumberConstruction, NumberDefinition, Provenance, ProvenanceSource, ValidatedNumber,
    ValueOrigin,
};

pub const CHACHA20_ALGORITHM: &str = "chacha20";
pub const CHACHA20_ALGORITHM_VERSION: &str = "rand_chacha-0.10.0";
pub const CONTROL_MAPPING_VERSION: &str = "sha256-subseed-little-endian-mask-v1";
pub const MAX_SAMPLES_PER_INPUT_V1: u32 = 4096;

/// Complete deterministic control-generation configuration.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlSpecification {
    pub algorithm: String,
    pub algorithm_version: String,
    pub seed_hex: String,
    pub samples_per_input: u32,
    pub mapping_version: String,
    pub rejection_policy: RejectionPolicy,
}

impl ControlSpecification {
    pub fn validate(&self) -> Result<[u8; 32], ControlError> {
        validate_exact("algorithm", &self.algorithm, CHACHA20_ALGORITHM)?;
        validate_exact(
            "algorithm_version",
            &self.algorithm_version,
            CHACHA20_ALGORITHM_VERSION,
        )?;
        validate_exact(
            "mapping_version",
            &self.mapping_version,
            CONTROL_MAPPING_VERSION,
        )?;
        if self.samples_per_input == 0 {
            return Err(ControlError::ZeroSampleSize);
        }
        if self.samples_per_input > MAX_SAMPLES_PER_INPUT_V1 {
            return Err(ControlError::SampleSizeTooLarge {
                requested: self.samples_per_input,
                maximum: MAX_SAMPLES_PER_INPUT_V1,
            });
        }
        if self.rejection_policy != RejectionPolicy::mvp_default() {
            return Err(ControlError::UnsupportedRejectionPolicy);
        }
        decode_seed(&self.seed_hex)
    }
}

/// The declared MVP population and ordered rejection rules.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RejectionPolicy {
    pub parity_population: String,
    pub reject_equal_to_special: bool,
    pub reject_duplicates_within_match: bool,
    pub rejection_order: RejectionOrder,
}

impl RejectionPolicy {
    pub fn mvp_default() -> Self {
        Self {
            parity_population: "both".into(),
            reject_equal_to_special: true,
            reject_duplicates_within_match: true,
            rejection_order: RejectionOrder::EqualityThenDuplicate,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RejectionOrder {
    EqualityThenDuplicate,
}

/// One accepted deterministic control and its exact match mapping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedControl {
    pub matched_special_input_id: String,
    pub replicate_index: u32,
    pub number: ValidatedNumber,
}

pub fn generate_controls(
    experiment_id: &str,
    special: &ValidatedNumber,
    specification: &ControlSpecification,
) -> Result<Vec<GeneratedControl>, ControlError> {
    let master_seed = specification.validate()?;
    let bit_length = special.value().get().significant_bits();
    ensure_population_capacity(bit_length, specification.samples_per_input)?;

    let byte_length = usize::try_from(bit_length.div_ceil(8))
        .map_err(|_| ControlError::BitLengthTooLarge { bit_length })?;
    let excess_bits = byte_length
        .checked_mul(8)
        .and_then(|bits| bits.checked_sub(bit_length as usize))
        .ok_or(ControlError::BitLengthTooLarge { bit_length })?;
    let mut accepted = HashSet::new();
    let requested = specification.samples_per_input as usize;
    accepted
        .try_reserve(requested)
        .map_err(|_| ControlError::AllocationFailed {
            context: "accepted control set",
            requested,
        })?;
    let mut controls = Vec::new();
    controls
        .try_reserve_exact(requested)
        .map_err(|_| ControlError::AllocationFailed {
            context: "control result vector",
            requested,
        })?;

    for replicate_index in 0..specification.samples_per_input {
        let subseed = control_subseed(
            master_seed,
            experiment_id,
            &special.definition().input_id,
            replicate_index,
        );
        let mut rng = ChaCha20Rng::from_seed(subseed);

        let value = loop {
            let mut bytes = Vec::new();
            bytes
                .try_reserve_exact(byte_length)
                .map_err(|_| ControlError::AllocationFailed {
                    context: "control candidate bytes",
                    requested: byte_length,
                })?;
            bytes.resize(byte_length, 0);
            rng.fill_bytes(&mut bytes);
            let most_significant = bytes
                .last_mut()
                .ok_or(ControlError::BitLengthTooLarge { bit_length })?;
            *most_significant &= u8::MAX >> excess_bits;
            *most_significant |= 1_u8 << (7 - excess_bits);

            let candidate = Integer::from_digits(&bytes, Order::Lsf);
            if candidate == *special.value().get() {
                continue;
            }
            if accepted.contains(&candidate) {
                continue;
            }
            break candidate;
        };

        accepted.insert(value.clone());
        let decimal_value = value.to_string();
        let input_id = format!(
            "control-{}-{:03}",
            special.definition().input_id,
            replicate_index + 1
        );
        let definition = NumberDefinition {
            schema_version: crate::NUMBER_DEFINITION_SCHEMA_VERSION,
            input_id,
            name: format!(
                "Matched control {} #{}",
                special.definition().name,
                replicate_index + 1
            ),
            family: "matched-control".into(),
            construction: NumberConstruction::Literal {
                value: decimal_value.clone(),
            },
            provenance: Provenance {
                origin: ValueOrigin::Generated,
                source_kind: ProvenanceSource::Local,
                source: "Collatz Lab deterministic control generator".into(),
                external_id: None,
                retrieval_date: None,
                imported_value_sha256: None,
                reconstruction_note: format!(
                    "Regenerate from experiment {experiment_id}, matched input {}, replicate {replicate_index}, {}, {}, seed {}, and the declared MVP rejection policy.",
                    special.definition().input_id,
                    specification.algorithm_version,
                    specification.mapping_version,
                    specification.seed_hex
                ),
            },
            declared_bit_length: bit_length,
            declared_decimal_digits: u32::try_from(decimal_value.len())
                .map_err(|_| ControlError::BitLengthTooLarge { bit_length })?,
        };
        let number = ValidatedNumber::validate(definition)
            .map_err(|source| ControlError::GeneratedDefinitionInvalid { source })?;

        controls.push(GeneratedControl {
            matched_special_input_id: special.definition().input_id.clone(),
            replicate_index,
            number,
        });
    }

    Ok(controls)
}

/// Deterministic control configuration or generation failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ControlError {
    UnsupportedValue {
        field: &'static str,
        found: String,
        expected: &'static str,
    },
    InvalidSeed,
    ZeroSampleSize,
    SampleSizeTooLarge {
        requested: u32,
        maximum: u32,
    },
    UnsupportedRejectionPolicy,
    ControlSpaceExhausted {
        bit_length: u32,
        requested: u32,
        available: u64,
    },
    BitLengthTooLarge {
        bit_length: u32,
    },
    AllocationFailed {
        context: &'static str,
        requested: usize,
    },
    GeneratedDefinitionInvalid {
        source: crate::NumberValidationError,
    },
}

impl fmt::Display for ControlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedValue {
                field,
                found,
                expected,
            } => write!(
                formatter,
                "unsupported {field} {found}; expected {expected}"
            ),
            Self::InvalidSeed => formatter.write_str("seed_hex must be 64 lowercase hex digits"),
            Self::ZeroSampleSize => formatter.write_str("samples_per_input must be at least 1"),
            Self::SampleSizeTooLarge { requested, maximum } => write!(
                formatter,
                "samples_per_input {requested} exceeds the version-1 maximum {maximum}"
            ),
            Self::UnsupportedRejectionPolicy => {
                formatter.write_str("control rejection policy is not the declared MVP policy")
            }
            Self::ControlSpaceExhausted {
                bit_length,
                requested,
                available,
            } => write!(
                formatter,
                "cannot draw {requested} distinct {bit_length}-bit controls after excluding the special value; only {available} are available"
            ),
            Self::BitLengthTooLarge { bit_length } => {
                write!(
                    formatter,
                    "control bit length {bit_length} is not addressable"
                )
            }
            Self::AllocationFailed { context, requested } => write!(
                formatter,
                "cannot reserve {requested} elements for {context}"
            ),
            Self::GeneratedDefinitionInvalid { source } => {
                write!(formatter, "generated control failed validation: {source}")
            }
        }
    }
}

impl std::error::Error for ControlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::GeneratedDefinitionInvalid { source } => Some(source),
            _ => None,
        }
    }
}

fn validate_exact(
    field: &'static str,
    found: &str,
    expected: &'static str,
) -> Result<(), ControlError> {
    if found == expected {
        Ok(())
    } else {
        Err(ControlError::UnsupportedValue {
            field,
            found: found.into(),
            expected,
        })
    }
}

fn decode_seed(seed_hex: &str) -> Result<[u8; 32], ControlError> {
    if seed_hex.len() != 64
        || !seed_hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ControlError::InvalidSeed);
    }

    let mut seed = [0_u8; 32];
    for (index, output) in seed.iter_mut().enumerate() {
        let offset = index * 2;
        *output = u8::from_str_radix(&seed_hex[offset..offset + 2], 16)
            .map_err(|_| ControlError::InvalidSeed)?;
    }
    Ok(seed)
}

fn control_subseed(
    master_seed: [u8; 32],
    experiment_id: &str,
    input_id: &str,
    replicate_index: u32,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"collatz-lab-control-v1\0");
    hasher.update(master_seed);
    hasher.update(b"\0");
    hasher.update(experiment_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(input_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(replicate_index.to_be_bytes());
    hasher.finalize().into()
}

fn ensure_population_capacity(bit_length: u32, requested: u32) -> Result<(), ControlError> {
    if bit_length <= 63 {
        let population = 1_u64 << (bit_length - 1);
        let available = population - 1;
        if u64::from(requested) > available {
            return Err(ControlError::ControlSpaceExhausted {
                bit_length,
                requested,
                available,
            });
        }
    }
    Ok(())
}
