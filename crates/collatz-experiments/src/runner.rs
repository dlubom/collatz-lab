use core::fmt;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use collatz_engine::{
    BigIntRunSummary, HybridRunSummary, HybridValue, PositiveU128, RunError, RunSummary,
    Termination, run, run_bigint, run_hybrid,
};
use sha2::{Digest, Sha256};

use crate::config::EXPERIMENT_PLAN_SCHEMA_VERSION;
use crate::{
    Catalog, CatalogError, EngineOutcome, EnginePolicy, ExperimentConfigError,
    ExperimentConfiguration, ExperimentPlan, ExperimentStatus, LabeledMetric, PlannedInput,
    ResultRecord, ValidatedNumber, ValidationState,
};

static RUN_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A complete local run and its line-oriented records.
#[derive(Clone, Debug)]
pub struct RunOutput {
    pub plan: ExperimentPlan,
    pub run_id: String,
    pub records: Vec<ResultRecord>,
}

impl RunOutput {
    pub fn write_jsonl(&self, path: impl AsRef<Path>) -> Result<(), RunnerError> {
        let path = path.as_ref();
        let file = File::create(path).map_err(|source| RunnerError::Io {
            operation: "create",
            path: path.to_path_buf(),
            source,
        })?;
        let mut writer = BufWriter::new(file);
        for record in &self.records {
            serde_json::to_writer(&mut writer, record).map_err(|source| RunnerError::Json {
                context: "result record".into(),
                message: source.to_string(),
            })?;
            writer.write_all(b"\n").map_err(|source| RunnerError::Io {
                operation: "write",
                path: path.to_path_buf(),
                source,
            })?;
        }
        writer.flush().map_err(|source| RunnerError::Io {
            operation: "flush",
            path: path.to_path_buf(),
            source,
        })
    }
}

/// Loads and materializes one configuration using its declared catalog.
pub fn materialize_configuration(
    configuration_path: impl AsRef<Path>,
) -> Result<ExperimentPlan, RunnerError> {
    let configuration =
        ExperimentConfiguration::load(configuration_path).map_err(RunnerError::Configuration)?;
    let catalog = Catalog::load_jsonl(&configuration.catalog_path).map_err(RunnerError::Catalog)?;
    configuration
        .materialize(&catalog)
        .map_err(RunnerError::Configuration)
}

/// Loads, materializes, validates, and executes one configuration.
pub fn run_configuration(configuration_path: impl AsRef<Path>) -> Result<RunOutput, RunnerError> {
    let plan = materialize_configuration(configuration_path)?;
    execute_plan(plan)
}

fn execute_plan(plan: ExperimentPlan) -> Result<RunOutput, RunnerError> {
    validate_plan(&plan)?;
    let run_id = create_run_id(&plan.configuration_id);
    let mut records = Vec::with_capacity(plan.inputs.len());

    for (index, input) in plan.inputs.iter().enumerate() {
        let validated = ValidatedNumber::validate(input.definition.clone())
            .map_err(RunnerError::InvalidPlannedInput)?;
        if validated.decimal_value() != input.decimal_value {
            return Err(RunnerError::PlanValueMismatch {
                input_id: input.definition.input_id.clone(),
                planned: input.decimal_value.clone(),
                reconstructed: validated.decimal_value(),
            });
        }
        let observation_index = u32::try_from(index).map_err(|_| RunnerError::TooManyInputs)?;
        records.push(execute_input(
            &plan,
            input,
            &validated,
            &run_id,
            observation_index,
        ));
    }

    Ok(RunOutput {
        plan,
        run_id,
        records,
    })
}

fn validate_plan(plan: &ExperimentPlan) -> Result<(), RunnerError> {
    if plan.schema_version != EXPERIMENT_PLAN_SCHEMA_VERSION {
        return Err(RunnerError::PlanSchemaVersion {
            found: plan.schema_version,
        });
    }
    let expected_id = plan
        .configuration
        .configuration_id()
        .map_err(RunnerError::Configuration)?;
    if expected_id != plan.configuration_id {
        return Err(RunnerError::ConfigurationIdMismatch {
            declared: plan.configuration_id.clone(),
            actual: expected_id,
        });
    }
    Ok(())
}

fn execute_input(
    plan: &ExperimentPlan,
    input: &PlannedInput,
    validated: &ValidatedNumber,
    run_id: &str,
    observation_index: u32,
) -> ResultRecord {
    let start = Instant::now();
    let observation = match plan.configuration.engine_policy {
        EnginePolicy::Reference => {
            execute_reference(validated, plan.configuration.limits.classical_step_limit)
        }
        EnginePolicy::Bigint => Observation::from_bigint(run_bigint(
            validated.value().clone(),
            plan.configuration.limits.classical_step_limit,
        )),
        EnginePolicy::Hybrid => {
            execute_hybrid(validated, plan.configuration.limits.classical_step_limit)
        }
    };
    let elapsed_nanoseconds = u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let result_id = create_result_id(run_id, observation_index, &input.definition.input_id);

    ResultRecord {
        schema_version: crate::result::RESULT_SCHEMA_VERSION,
        result_id,
        experiment_id: plan.configuration.experiment_id.clone(),
        configuration_id: plan.configuration_id.clone(),
        run_id: run_id.into(),
        observation_index,
        input: input.into(),
        engine_policy: plan.configuration.engine_policy,
        limits: plan.configuration.limits.clone(),
        status: observation.status,
        engine_outcome: observation.engine_outcome,
        completed_classical_steps: observation.completed_classical_steps,
        classical_steps_to_one: observation.classical_steps_to_one,
        observed_peak: observation.observed_peak,
        first_descent: observation.first_descent,
        last_value: observation.last_value,
        elapsed_nanoseconds,
        promotion_count: observation.promotion_count,
        program_commit: plan.configuration.program_commit.clone(),
        validation_state: observation.validation_state,
        validation_method: "catalog-reconstruction-v1; declared-engine-policy-v1".into(),
    }
}

fn execute_reference(validated: &ValidatedNumber, limit: u64) -> Observation {
    let Some(start_value) = validated.value().get().to_u128() else {
        return Observation::not_representable();
    };
    let Ok(start) = PositiveU128::new(start_value) else {
        return Observation::not_representable();
    };

    match run(start, limit) {
        Ok(summary) => Observation::from_reference(summary),
        Err(error) => Observation::from_reference_overflow(error),
    }
}

fn execute_hybrid(validated: &ValidatedNumber, limit: u64) -> Observation {
    let Some(start_value) = validated.value().get().to_u128() else {
        return Observation::not_representable();
    };
    let Ok(start) = PositiveU128::new(start_value) else {
        return Observation::not_representable();
    };
    Observation::from_hybrid(run_hybrid(start, limit))
}

struct Observation {
    status: ExperimentStatus,
    engine_outcome: EngineOutcome,
    completed_classical_steps: u64,
    classical_steps_to_one: LabeledMetric<u64>,
    observed_peak: LabeledMetric<String>,
    first_descent: LabeledMetric<u64>,
    last_value: Option<String>,
    promotion_count: u8,
    validation_state: ValidationState,
}

impl Observation {
    fn from_reference(summary: RunSummary) -> Self {
        Self::from_normal(
            summary.termination,
            summary.completed_classical_steps,
            summary.observed_peak.get().to_string(),
            summary.first_descent_step,
            summary.last.get().to_string(),
            0,
        )
    }

    fn from_bigint(summary: BigIntRunSummary) -> Self {
        Self::from_normal(
            summary.termination,
            summary.completed_classical_steps,
            summary.observed_peak.get().to_string(),
            summary.first_descent_step,
            summary.last.get().to_string(),
            0,
        )
    }

    fn from_hybrid(summary: HybridRunSummary) -> Self {
        Self::from_normal(
            summary.termination,
            summary.completed_classical_steps,
            hybrid_decimal(&summary.observed_peak),
            summary.first_descent_step,
            hybrid_decimal(&summary.last),
            summary.promotion_count,
        )
    }

    fn from_normal(
        termination: Termination,
        completed_classical_steps: u64,
        observed_peak: String,
        first_descent_step: Option<u64>,
        last_value: String,
        promotion_count: u8,
    ) -> Self {
        let complete = termination == Termination::ReachedOne;
        Self {
            status: if complete {
                ExperimentStatus::ReachedOne
            } else {
                ExperimentStatus::StepLimitReached
            },
            engine_outcome: EngineOutcome::Completed,
            completed_classical_steps,
            classical_steps_to_one: if complete {
                LabeledMetric::complete(Some(completed_classical_steps))
            } else {
                LabeledMetric::unavailable()
            },
            observed_peak: if complete {
                LabeledMetric::complete(Some(observed_peak))
            } else {
                LabeledMetric::prefix(Some(observed_peak))
            },
            first_descent: if complete || first_descent_step.is_some() {
                LabeledMetric::complete(first_descent_step)
            } else {
                LabeledMetric::prefix(None)
            },
            last_value: Some(last_value),
            promotion_count,
            validation_state: ValidationState::Validated,
        }
    }

    fn from_reference_overflow(error: RunError) -> Self {
        let first_descent = if error.progress.first_descent_step.is_some() {
            LabeledMetric::complete(error.progress.first_descent_step)
        } else {
            LabeledMetric::prefix(None)
        };
        Self {
            status: ExperimentStatus::VerificationFailed,
            engine_outcome: EngineOutcome::ReferenceArithmeticOverflow,
            completed_classical_steps: error.progress.completed_classical_steps,
            classical_steps_to_one: LabeledMetric::unavailable(),
            observed_peak: LabeledMetric::prefix(Some(
                error.progress.observed_peak.get().to_string(),
            )),
            first_descent,
            last_value: Some(error.progress.last.get().to_string()),
            promotion_count: 0,
            validation_state: ValidationState::VerificationFailed,
        }
    }

    fn not_representable() -> Self {
        Self {
            status: ExperimentStatus::VerificationFailed,
            engine_outcome: EngineOutcome::InputNotRepresentable,
            completed_classical_steps: 0,
            classical_steps_to_one: LabeledMetric::unavailable(),
            observed_peak: LabeledMetric::unavailable(),
            first_descent: LabeledMetric::unavailable(),
            last_value: None,
            promotion_count: 0,
            validation_state: ValidationState::VerificationFailed,
        }
    }
}

fn hybrid_decimal(value: &HybridValue) -> String {
    value.to_integer().to_string()
}

fn create_run_id(configuration_id: &str) -> String {
    let counter = RUN_COUNTER.fetch_add(1, Ordering::Relaxed);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(b"collatz-lab-run-v1\0");
    hasher.update(configuration_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(std::process::id().to_be_bytes());
    hasher.update(timestamp.to_be_bytes());
    hasher.update(counter.to_be_bytes());
    format!("run-{}", digest_hex(hasher))
}

fn create_result_id(run_id: &str, observation_index: u32, input_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"collatz-lab-result-v1\0");
    hasher.update(run_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(observation_index.to_be_bytes());
    hasher.update(b"\0");
    hasher.update(input_id.as_bytes());
    format!("result-{}", digest_hex(hasher))
}

fn digest_hex(hasher: Sha256) -> String {
    let digest = hasher.finalize();
    let mut output = String::with_capacity(64);
    for byte in digest {
        use core::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

/// Configuration, catalog, plan, execution, or result-output failure.
#[derive(Debug)]
pub enum RunnerError {
    Configuration(ExperimentConfigError),
    Catalog(CatalogError),
    InvalidPlannedInput(crate::NumberValidationError),
    PlanValueMismatch {
        input_id: String,
        planned: String,
        reconstructed: String,
    },
    PlanSchemaVersion {
        found: u32,
    },
    ConfigurationIdMismatch {
        declared: String,
        actual: String,
    },
    TooManyInputs,
    Json {
        context: String,
        message: String,
    },
    Io {
        operation: &'static str,
        path: std::path::PathBuf,
        source: std::io::Error,
    },
}

impl RunnerError {
    pub const fn status_code(&self) -> &'static str {
        match self {
            Self::Configuration(_) | Self::Catalog(_) | Self::InvalidPlannedInput(_) => {
                "invalid_input"
            }
            _ => "verification_failed",
        }
    }
}

impl fmt::Display for RunnerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration(source) => write!(formatter, "configuration: {source}"),
            Self::Catalog(source) => write!(formatter, "catalog: {source}"),
            Self::InvalidPlannedInput(source) => {
                write!(formatter, "invalid planned input: {source}")
            }
            Self::PlanValueMismatch {
                input_id,
                planned,
                reconstructed,
            } => write!(
                formatter,
                "plan value mismatch for {input_id}: planned {planned}, reconstructed {reconstructed}"
            ),
            Self::PlanSchemaVersion { found } => {
                write!(formatter, "unsupported plan schema version {found}")
            }
            Self::ConfigurationIdMismatch { declared, actual } => write!(
                formatter,
                "configuration ID mismatch: declared {declared}, reconstructed {actual}"
            ),
            Self::TooManyInputs => {
                formatter.write_str("plan has more inputs than result IDs support")
            }
            Self::Json { context, message } => {
                write!(formatter, "cannot serialize {context}: {message}")
            }
            Self::Io {
                operation,
                path,
                source,
            } => write!(formatter, "cannot {operation} {}: {source}", path.display()),
        }
    }
}

impl std::error::Error for RunnerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Configuration(source) => Some(source),
            Self::Catalog(source) => Some(source),
            Self::InvalidPlannedInput(source) => Some(source),
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
