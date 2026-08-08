use serde::{Deserialize, Serialize};

use crate::{EnginePolicy, ExperimentLimits, InputRole, NumberDefinition, PlannedInput};

pub const RESULT_SCHEMA_VERSION: u32 = 1;

/// One JSONL observation with all identifiers and reconstructed provenance.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultRecord {
    pub schema_version: u32,
    pub result_id: String,
    pub experiment_id: String,
    pub configuration_id: String,
    pub run_id: String,
    pub observation_index: u32,
    pub input: ResultInput,
    pub engine_policy: EnginePolicy,
    pub limits: ExperimentLimits,
    pub status: ExperimentStatus,
    pub engine_outcome: EngineOutcome,
    pub completed_classical_steps: u64,
    pub classical_steps_to_one: LabeledMetric<u64>,
    pub observed_peak: LabeledMetric<String>,
    pub first_descent: LabeledMetric<u64>,
    pub last_value: Option<String>,
    pub elapsed_nanoseconds: u64,
    pub promotion_count: u8,
    pub program_source_sha256: String,
    pub program_source_dirty: bool,
    pub validation_state: ValidationState,
    pub validation_method: String,
}

impl ResultRecord {
    /// Marks an observation conservatively until independent reproduction.
    pub fn mark_needs_reproduction(&mut self) {
        self.validation_state = ValidationState::NeedsReproduction;
    }
}

/// The input definition and match mapping repeated in the result record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultInput {
    pub role: InputRole,
    pub matched_special_input_id: Option<String>,
    pub replicate_index: Option<u32>,
    pub definition: NumberDefinition,
    pub decimal_value: String,
}

impl From<&PlannedInput> for ResultInput {
    fn from(input: &PlannedInput) -> Self {
        Self {
            role: input.role,
            matched_special_input_id: input.matched_special_input_id.clone(),
            replicate_index: input.replicate_index,
            definition: input.definition.clone(),
            decimal_value: input.decimal_value.clone(),
        }
    }
}

/// Controlled experiment termination vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExperimentStatus {
    ReachedOne,
    ReachedVerifiedBound,
    StepLimitReached,
    TimeLimitReached,
    ResourceLimitReached,
    EngineError,
    InvalidInput,
    VerificationFailed,
}

/// Engine-specific completion detail kept separate from experiment status.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineOutcome {
    Completed,
    ReferenceArithmeticOverflow,
    InputNotRepresentable,
}

/// Whether a metric describes a complete trajectory, observed prefix, or no value.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricCompleteness {
    Complete,
    Prefix,
    Unavailable,
    Derived,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabeledMetric<T> {
    pub completeness: MetricCompleteness,
    pub value: Option<T>,
}

impl<T> LabeledMetric<T> {
    pub fn complete(value: Option<T>) -> Self {
        Self {
            completeness: MetricCompleteness::Complete,
            value,
        }
    }

    pub fn prefix(value: Option<T>) -> Self {
        Self {
            completeness: MetricCompleteness::Prefix,
            value,
        }
    }

    pub fn unavailable() -> Self {
        Self {
            completeness: MetricCompleteness::Unavailable,
            value: None,
        }
    }
}

/// Research review state; exceptional observations cannot start as reproduced.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationState {
    Validated,
    NeedsReproduction,
    VerificationFailed,
}
