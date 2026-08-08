#![forbid(unsafe_code)]

//! Reproducible Collatz input, control, configuration, and result contracts.

pub mod catalog;
pub mod config;
pub mod controls;
pub mod number;
pub mod result;
pub mod runner;

pub use catalog::{Catalog, CatalogError};
pub use config::{
    EXPERIMENT_CONFIG_SCHEMA_VERSION, EnginePolicy, ExperimentConfigError, ExperimentConfiguration,
    ExperimentLimits, ExperimentPlan, InputRole, MetricName, PlannedInput, VerifiedBoundReference,
};
pub use controls::{
    CHACHA20_ALGORITHM, CHACHA20_ALGORITHM_VERSION, CONTROL_MAPPING_VERSION, ControlError,
    ControlSpecification, GeneratedControl, RejectionOrder, RejectionPolicy, generate_controls,
};
pub use number::{
    NUMBER_DEFINITION_SCHEMA_VERSION, NumberConstruction, NumberDefinition, NumberValidationError,
    Provenance, ValidatedNumber, ValueOrigin,
};
pub use result::{
    EngineOutcome, ExperimentStatus, LabeledMetric, MetricCompleteness, ResultRecord,
    ValidationState,
};
pub use runner::{RunOutput, RunnerError, materialize_configuration, run_configuration};
