#![forbid(unsafe_code)]

//! Reproducible Collatz input, control, configuration, and result contracts.

pub mod catalog;
pub mod config;
pub mod controls;
pub mod number;
mod program;
pub mod result;
pub mod runner;

pub use catalog::{Catalog, CatalogError};
pub use config::{
    EXPERIMENT_CONFIG_SCHEMA_VERSION, EnginePolicy, ExperimentConfigError, ExperimentConfiguration,
    ExperimentLimits, ExperimentPlan, InputRole, MAX_OBSERVATIONS_V1, MetricName, PlannedInput,
    VerifiedBoundReference,
};
pub use controls::{
    CHACHA20_ALGORITHM, CHACHA20_ALGORITHM_VERSION, CONTROL_MAPPING_VERSION, ControlError,
    ControlSpecification, GeneratedControl, MAX_SAMPLES_PER_INPUT_V1, RejectionOrder,
    RejectionPolicy, generate_controls,
};
pub use number::{
    MAX_FERMAT_INDEX_V1, NUMBER_DEFINITION_SCHEMA_VERSION, NumberConstruction, NumberDefinition,
    NumberValidationError, Provenance, ProvenanceSource, ValidatedNumber, ValueOrigin,
};
pub use program::{program_source_dirty, program_source_sha256};
pub use result::{
    EngineOutcome, ExperimentStatus, LabeledMetric, MetricCompleteness, ResultRecord,
    ValidationState,
};
pub use runner::{RunOutput, RunnerError, materialize_configuration, run_configuration};
