use core::fmt;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::number::hex_sha256;
use crate::{
    Catalog, ControlError, ControlSpecification, GeneratedControl, NumberDefinition,
    ValidatedNumber,
};

pub const EXPERIMENT_CONFIG_SCHEMA_VERSION: u32 = 1;
pub const EXPERIMENT_PLAN_SCHEMA_VERSION: u32 = 1;
pub const RESULT_FORMAT_VERSION: u32 = 1;

/// An exact, versioned experiment setup before materialization.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentConfiguration {
    pub schema_version: u32,
    pub experiment_id: String,
    pub name: String,
    pub catalog_path: String,
    pub input_ids: Vec<String>,
    pub engine_policy: EnginePolicy,
    pub limits: ExperimentLimits,
    pub metrics: Vec<MetricName>,
    pub primary_metric: MetricName,
    pub controls: Option<ControlSpecification>,
    pub verified_bound: Option<VerifiedBoundReference>,
    pub output_format_version: u32,
    pub program_commit: String,
}

impl ExperimentConfiguration {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ExperimentConfigError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| ExperimentConfigError::Io {
            operation: "read",
            path: path.to_path_buf(),
            source,
        })?;
        let configuration =
            serde_json::from_slice(&bytes).map_err(|source| ExperimentConfigError::Json {
                context: path.display().to_string(),
                message: source.to_string(),
            })?;
        Ok(configuration)
    }

    pub fn validate(&self) -> Result<(), ExperimentConfigError> {
        if self.schema_version != EXPERIMENT_CONFIG_SCHEMA_VERSION {
            return Err(ExperimentConfigError::UnsupportedSchemaVersion {
                found: self.schema_version,
            });
        }
        validate_identifier("experiment_id", &self.experiment_id)?;
        validate_nonempty("name", &self.name)?;
        validate_nonempty("catalog_path", &self.catalog_path)?;
        if self.input_ids.is_empty() {
            return Err(ExperimentConfigError::EmptyInputs);
        }
        let mut input_ids = HashSet::new();
        for input_id in &self.input_ids {
            validate_identifier("input_id", input_id)?;
            if !input_ids.insert(input_id) {
                return Err(ExperimentConfigError::DuplicateInputId {
                    input_id: input_id.clone(),
                });
            }
        }
        if self.metrics.is_empty() {
            return Err(ExperimentConfigError::EmptyMetrics);
        }
        let mut metrics = HashSet::new();
        for metric in &self.metrics {
            if !metrics.insert(*metric) {
                return Err(ExperimentConfigError::DuplicateMetric { metric: *metric });
            }
        }
        if !metrics.contains(&self.primary_metric) {
            return Err(ExperimentConfigError::PrimaryMetricNotSelected {
                metric: self.primary_metric,
            });
        }
        if let Some(controls) = &self.controls {
            controls
                .validate()
                .map_err(ExperimentConfigError::InvalidControls)?;
        }
        if self.verified_bound.is_some() {
            return Err(ExperimentConfigError::VerifiedBoundNotExecutable);
        }
        if self.limits.time_limit_ms.is_some() || self.limits.resource_limit_bytes.is_some() {
            return Err(ExperimentConfigError::OperationalLimitNotExecutable);
        }
        if self.output_format_version != RESULT_FORMAT_VERSION {
            return Err(ExperimentConfigError::UnsupportedOutputFormatVersion {
                found: self.output_format_version,
            });
        }
        validate_commit(&self.program_commit)?;
        Ok(())
    }

    pub fn configuration_id(&self, catalog: &Catalog) -> Result<String, ExperimentConfigError> {
        let selected_numbers = self.selected_numbers(catalog)?;
        let selected_definitions: Vec<_> = selected_numbers
            .iter()
            .map(|number| number.definition().clone())
            .collect();
        self.configuration_id_for_definitions(&selected_definitions)
    }

    pub(crate) fn configuration_id_for_definitions(
        &self,
        selected_definitions: &[NumberDefinition],
    ) -> Result<String, ExperimentConfigError> {
        self.validate()?;
        if selected_definitions.len() != self.input_ids.len()
            || self
                .input_ids
                .iter()
                .zip(selected_definitions)
                .any(|(input_id, definition)| input_id != &definition.input_id)
        {
            return Err(ExperimentConfigError::SelectedDefinitionsMismatch);
        }
        let identity = ConfigurationIdentity {
            configuration: self,
            selected_input_definitions: selected_definitions,
        };
        let bytes =
            serde_json::to_vec(&identity).map_err(|source| ExperimentConfigError::Json {
                context: "canonical experiment identity".into(),
                message: source.to_string(),
            })?;
        Ok(hex_sha256(&bytes))
    }

    pub fn materialize(&self, catalog: &Catalog) -> Result<ExperimentPlan, ExperimentConfigError> {
        let special_numbers = self.selected_numbers(catalog)?;
        let selected_definitions: Vec<_> = special_numbers
            .iter()
            .map(|number| number.definition().clone())
            .collect();
        let configuration_id = self.configuration_id_for_definitions(&selected_definitions)?;
        let mut inputs = Vec::new();

        for number in &special_numbers {
            inputs.push(PlannedInput::special(
                number.definition().clone(),
                number.decimal_value(),
            ));
        }

        if let Some(control_specification) = &self.controls {
            for special in special_numbers {
                let generated =
                    crate::generate_controls(&self.experiment_id, special, control_specification)
                        .map_err(ExperimentConfigError::InvalidControls)?;
                inputs.extend(generated.into_iter().map(PlannedInput::control));
            }
        }

        Ok(ExperimentPlan {
            schema_version: EXPERIMENT_PLAN_SCHEMA_VERSION,
            configuration_id,
            configuration: self.clone(),
            inputs,
        })
    }

    fn selected_numbers<'a>(
        &self,
        catalog: &'a Catalog,
    ) -> Result<Vec<&'a ValidatedNumber>, ExperimentConfigError> {
        self.validate()?;
        self.input_ids
            .iter()
            .map(|input_id| {
                catalog
                    .get(input_id)
                    .ok_or_else(|| ExperimentConfigError::CatalogInputMissing {
                        input_id: input_id.clone(),
                    })
            })
            .collect()
    }
}

#[derive(Serialize)]
struct ConfigurationIdentity<'a> {
    configuration: &'a ExperimentConfiguration,
    selected_input_definitions: &'a [NumberDefinition],
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnginePolicy {
    Reference,
    Bigint,
    Hybrid,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentLimits {
    pub classical_step_limit: u64,
    pub time_limit_ms: Option<u64>,
    pub resource_limit_bytes: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricName {
    ClassicalSteps,
    ObservedPeak,
    FirstDescent,
}

impl fmt::Display for MetricName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::ClassicalSteps => "classical_steps",
            Self::ObservedPeak => "observed_peak",
            Self::FirstDescent => "first_descent",
        };
        formatter.write_str(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifiedBoundReference {
    pub inclusive_upper_bound: String,
    pub source: String,
    pub version: String,
    pub retrieval_date: String,
}

/// A canonical plan contains no run-specific identifier or clock value.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentPlan {
    pub schema_version: u32,
    pub configuration_id: String,
    pub configuration: ExperimentConfiguration,
    pub inputs: Vec<PlannedInput>,
}

impl ExperimentPlan {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ExperimentConfigError> {
        let mut bytes =
            serde_json::to_vec_pretty(self).map_err(|source| ExperimentConfigError::Json {
                context: "canonical experiment plan".into(),
                message: source.to_string(),
            })?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    pub fn write(&self, path: impl AsRef<Path>) -> Result<(), ExperimentConfigError> {
        let path = path.as_ref();
        fs::write(path, self.canonical_bytes()?).map_err(|source| ExperimentConfigError::Io {
            operation: "write",
            path: path.to_path_buf(),
            source,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputRole {
    Special,
    Control,
}

/// One ordered, fully reconstructed plan input.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlannedInput {
    pub role: InputRole,
    pub matched_special_input_id: Option<String>,
    pub replicate_index: Option<u32>,
    pub definition: NumberDefinition,
    pub decimal_value: String,
}

impl PlannedInput {
    fn special(definition: NumberDefinition, decimal_value: String) -> Self {
        Self {
            role: InputRole::Special,
            matched_special_input_id: None,
            replicate_index: None,
            definition,
            decimal_value,
        }
    }

    fn control(control: GeneratedControl) -> Self {
        Self {
            role: InputRole::Control,
            matched_special_input_id: Some(control.matched_special_input_id),
            replicate_index: Some(control.replicate_index),
            definition: control.number.definition().clone(),
            decimal_value: control.number.decimal_value(),
        }
    }
}

/// A configuration, plan, or canonical serialization failure.
#[derive(Debug)]
pub enum ExperimentConfigError {
    Io {
        operation: &'static str,
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    Json {
        context: String,
        message: String,
    },
    UnsupportedSchemaVersion {
        found: u32,
    },
    UnsupportedOutputFormatVersion {
        found: u32,
    },
    EmptyField {
        field: &'static str,
    },
    InvalidIdentifier {
        field: &'static str,
        value: String,
    },
    EmptyInputs,
    DuplicateInputId {
        input_id: String,
    },
    EmptyMetrics,
    DuplicateMetric {
        metric: MetricName,
    },
    PrimaryMetricNotSelected {
        metric: MetricName,
    },
    InvalidControls(ControlError),
    VerifiedBoundNotExecutable,
    OperationalLimitNotExecutable,
    InvalidProgramCommit {
        value: String,
    },
    CatalogInputMissing {
        input_id: String,
    },
    SelectedDefinitionsMismatch,
}

impl fmt::Display for ExperimentConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io {
                operation,
                path,
                source,
            } => write!(formatter, "cannot {operation} {}: {source}", path.display()),
            Self::Json { context, message } => write!(formatter, "invalid {context}: {message}"),
            Self::UnsupportedSchemaVersion { found } => {
                write!(formatter, "unsupported experiment schema version {found}")
            }
            Self::UnsupportedOutputFormatVersion { found } => {
                write!(formatter, "unsupported result format version {found}")
            }
            Self::EmptyField { field } => write!(formatter, "{field} must not be empty"),
            Self::InvalidIdentifier { field, value } => {
                write!(formatter, "invalid {field} identifier {value}")
            }
            Self::EmptyInputs => formatter.write_str("experiment must select at least one input"),
            Self::DuplicateInputId { input_id } => {
                write!(formatter, "duplicate experiment input_id {input_id}")
            }
            Self::EmptyMetrics => formatter.write_str("experiment must select at least one metric"),
            Self::DuplicateMetric { metric } => write!(formatter, "duplicate metric {metric}"),
            Self::PrimaryMetricNotSelected { metric } => {
                write!(formatter, "primary metric {metric} is not in metrics")
            }
            Self::InvalidControls(source) => write!(formatter, "invalid controls: {source}"),
            Self::VerifiedBoundNotExecutable => formatter.write_str(
                "verified-bound execution is not implemented in PBI-004; omit verified_bound",
            ),
            Self::OperationalLimitNotExecutable => formatter.write_str(
                "time and resource limits are schema-reserved but not executable in PBI-004",
            ),
            Self::InvalidProgramCommit { value } => {
                write!(
                    formatter,
                    "program_commit must be 40 lowercase hex digits: {value}"
                )
            }
            Self::CatalogInputMissing { input_id } => {
                write!(formatter, "catalog does not contain input_id {input_id}")
            }
            Self::SelectedDefinitionsMismatch => formatter
                .write_str("selected input definitions do not match configured input_ids in order"),
        }
    }
}

impl std::error::Error for ExperimentConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::InvalidControls(source) => Some(source),
            _ => None,
        }
    }
}

fn validate_identifier(field: &'static str, value: &str) -> Result<(), ExperimentConfigError> {
    if value.is_empty()
        || !value.split('-').all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
    {
        Err(ExperimentConfigError::InvalidIdentifier {
            field,
            value: value.into(),
        })
    } else {
        Ok(())
    }
}

fn validate_nonempty(field: &'static str, value: &str) -> Result<(), ExperimentConfigError> {
    if value.trim().is_empty() {
        Err(ExperimentConfigError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn validate_commit(value: &str) -> Result<(), ExperimentConfigError> {
    if value.len() != 40
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Err(ExperimentConfigError::InvalidProgramCommit {
            value: value.into(),
        })
    } else {
        Ok(())
    }
}
