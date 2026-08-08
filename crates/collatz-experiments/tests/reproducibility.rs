use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use collatz_experiments::{
    Catalog, ControlError, ControlSpecification, EngineOutcome, EnginePolicy,
    ExperimentConfigError, ExperimentConfiguration, ExperimentStatus, InputRole,
    MAX_OBSERVATIONS_V1, MAX_SAMPLES_PER_INPUT_V1, MetricCompleteness, NumberConstruction,
    NumberDefinition, Provenance, ProvenanceSource, RejectionPolicy, ValidatedNumber,
    ValidationState, ValueOrigin, generate_controls, program_source_dirty, program_source_sha256,
    run_configuration,
};
use proptest::prelude::*;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn repository_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn reviewed_catalog() -> Catalog {
    Catalog::load_jsonl(repository_path("catalog/inputs-v1.jsonl"))
        .expect("the reviewed catalog must validate")
}

fn load_configuration(relative: &str) -> ExperimentConfiguration {
    ExperimentConfiguration::load(repository_path(relative)).expect("fixture config must parse")
}

fn temp_path(suffix: &str) -> TempPath {
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "collatz-lab-{}-{counter}-{suffix}",
        std::process::id()
    ));
    TempPath(path)
}

struct TempPath(PathBuf);

impl TempPath {
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempPath {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn write_configuration(configuration: &ExperimentConfiguration) -> TempPath {
    let path = temp_path("experiment.json");
    let bytes = serde_json::to_vec_pretty(configuration).expect("fixture config serializes");
    std::fs::write(path.path(), bytes).expect("fixture config writes");
    path
}

fn runnable_fixture(relative: &str) -> TempPath {
    let mut configuration = load_configuration(relative);
    configuration.catalog_path = repository_path("catalog/inputs-v1.jsonl")
        .to_string_lossy()
        .into_owned();
    write_configuration(&configuration)
}

fn all_catalog_configuration(engine_policy: EnginePolicy) -> ExperimentConfiguration {
    let mut configuration = load_configuration("experiments/EXP-001.json");
    configuration.catalog_path = repository_path("catalog/inputs-v1.jsonl")
        .to_string_lossy()
        .into_owned();
    configuration.input_ids = reviewed_catalog()
        .entries()
        .iter()
        .map(|entry| entry.definition().input_id.clone())
        .collect();
    configuration.engine_policy = engine_policy;
    configuration.limits.classical_step_limit = 10_000;
    configuration
}

#[test]
fn exp002_plan_is_byte_identical_and_pins_the_first_control_word() {
    let catalog = reviewed_catalog();
    let configuration = load_configuration("experiments/EXP-002.json");
    let first = configuration
        .materialize(&catalog)
        .expect("first plan materializes");
    let second = configuration
        .materialize(&catalog)
        .expect("second plan materializes");

    assert_eq!(
        first.canonical_bytes().unwrap(),
        second.canonical_bytes().unwrap()
    );
    assert_eq!(first.configuration_id, second.configuration_id);
    assert_eq!(first.inputs.len(), 27);

    let first_control = first
        .inputs
        .iter()
        .find(|input| input.definition.input_id == "control-mersenne-5-001")
        .expect("pinned control exists");
    assert_eq!(first_control.decimal_value, "18");
    assert_eq!(first_control.replicate_index, Some(0));
    assert_eq!(
        first_control.matched_special_input_id.as_deref(),
        Some("mersenne-5")
    );
}

#[test]
fn configuration_rejects_a_program_source_hash_not_embedded_in_the_build() {
    let catalog = reviewed_catalog();
    let mut configuration = load_configuration("experiments/EXP-001.json");
    configuration.program_source_sha256 = "0".repeat(64);

    assert!(matches!(
        configuration.materialize(&catalog),
        Err(ExperimentConfigError::ProgramSourceSha256Mismatch { declared, built })
            if declared == "0".repeat(64) && built == program_source_sha256()
    ));
}

#[test]
fn results_take_program_provenance_from_the_built_library() {
    let configuration_path = runnable_fixture("experiments/EXP-001.json");
    let run = run_configuration(configuration_path.path()).expect("EXP-001 succeeds");

    assert!(run.records.iter().all(|record| {
        record.program_source_sha256 == program_source_sha256()
            && record.program_source_dirty == program_source_dirty()
    }));
}

#[test]
fn configuration_identity_binds_selected_catalog_definitions() {
    let catalog = reviewed_catalog();
    let configuration = load_configuration("experiments/EXP-001.json");
    let original = configuration
        .materialize(&catalog)
        .expect("original plan materializes");
    let mut definitions: Vec<_> = catalog
        .entries()
        .iter()
        .map(|entry| entry.definition().clone())
        .collect();
    let selected = definitions
        .iter_mut()
        .find(|definition| definition.input_id == "literal-1")
        .expect("selected definition exists");
    selected.provenance.reconstruction_note =
        "Equivalent reconstruction note whose catalog drift must change identity.".into();
    let drifted_catalog =
        Catalog::from_definitions(definitions).expect("drifted catalog validates");
    let drifted = configuration
        .materialize(&drifted_catalog)
        .expect("drifted plan materializes");

    assert_eq!(
        original.inputs[0].decimal_value,
        drifted.inputs[0].decimal_value
    );
    assert_ne!(original.configuration_id, drifted.configuration_id);
}

#[test]
fn every_control_matches_bits_and_obeys_equality_and_duplicate_rejections() {
    let catalog = reviewed_catalog();
    let plan = load_configuration("experiments/EXP-002.json")
        .materialize(&catalog)
        .expect("plan materializes");
    let specials: HashMap<_, _> = plan
        .inputs
        .iter()
        .filter(|input| input.role == InputRole::Special)
        .map(|input| {
            (
                input.definition.input_id.as_str(),
                (&input.decimal_value, input.definition.declared_bit_length),
            )
        })
        .collect();
    let mut matched_values: HashMap<&str, HashSet<&str>> = HashMap::new();

    for control in plan
        .inputs
        .iter()
        .filter(|input| input.role == InputRole::Control)
    {
        let matched_id = control
            .matched_special_input_id
            .as_deref()
            .expect("control has a match");
        let (special_value, bit_length) = specials[matched_id];
        assert_eq!(control.definition.declared_bit_length, bit_length);
        assert_ne!(&control.decimal_value, special_value);
        assert!(
            matched_values
                .entry(matched_id)
                .or_default()
                .insert(&control.decimal_value),
            "control duplicate for {matched_id}"
        );
    }
    assert_eq!(matched_values.values().map(HashSet::len).sum::<usize>(), 24);
}

#[test]
fn version_one_rejects_an_unbounded_control_sample_before_allocation() {
    let catalog = reviewed_catalog();
    let special = catalog.get("mersenne-13").expect("fixture input exists");
    let mut specification = load_configuration("experiments/EXP-002.json")
        .controls
        .expect("fixture controls exist");
    specification.samples_per_input = MAX_SAMPLES_PER_INPUT_V1 + 1;

    assert_eq!(
        generate_controls("exp-002", special, &specification),
        Err(ControlError::SampleSizeTooLarge {
            requested: MAX_SAMPLES_PER_INPUT_V1 + 1,
            maximum: MAX_SAMPLES_PER_INPUT_V1,
        })
    );
}

#[test]
fn version_one_rejects_too_many_total_observations_before_materialization() {
    let mut configuration = load_configuration("experiments/EXP-002.json");
    configuration.input_ids = vec![
        "literal-1".into(),
        "literal-2".into(),
        "literal-3".into(),
        "literal-27".into(),
    ];
    configuration
        .controls
        .as_mut()
        .expect("fixture controls exist")
        .samples_per_input = MAX_SAMPLES_PER_INPUT_V1;
    let requested = 4 * (MAX_SAMPLES_PER_INPUT_V1 as usize + 1);

    assert!(matches!(
        configuration.validate(),
        Err(ExperimentConfigError::TooManyObservations { requested: actual, maximum })
            if actual == requested && maximum == MAX_OBSERVATIONS_V1
    ));
}

#[test]
fn exp002_run_writes_27_consistent_and_uniquely_identified_results() {
    let configuration_path = runnable_fixture("experiments/EXP-002.json");
    let run = run_configuration(configuration_path.path()).expect("EXP-002 succeeds");
    let configuration_ids: HashSet<_> = run
        .records
        .iter()
        .map(|record| record.configuration_id.as_str())
        .collect();
    let run_ids: HashSet<_> = run
        .records
        .iter()
        .map(|record| record.run_id.as_str())
        .collect();
    let result_ids: HashSet<_> = run
        .records
        .iter()
        .map(|record| record.result_id.as_str())
        .collect();

    assert_eq!(run.records.len(), 27);
    assert_eq!(
        configuration_ids,
        HashSet::from([run.plan.configuration_id.as_str()])
    );
    assert_eq!(run_ids, HashSet::from([run.run_id.as_str()]));
    assert_eq!(result_ids.len(), 27);
    assert!(
        run.records
            .iter()
            .all(|record| record.status == ExperimentStatus::ReachedOne)
    );
}

#[test]
fn reruns_keep_configuration_identity_and_create_distinct_run_and_result_ids() {
    let configuration_path = runnable_fixture("experiments/EXP-001.json");
    let first = run_configuration(configuration_path.path()).expect("first run succeeds");
    let second = run_configuration(configuration_path.path()).expect("second run succeeds");

    assert_eq!(first.plan.configuration_id, second.plan.configuration_id);
    assert_ne!(first.run_id, second.run_id);
    assert_eq!(first.records.len(), second.records.len());
    assert!(
        first
            .records
            .iter()
            .all(|record| record.run_id == first.run_id)
    );
    assert!(
        second
            .records
            .iter()
            .all(|record| record.run_id == second.run_id)
    );
    for (left, right) in first.records.iter().zip(&second.records) {
        assert_ne!(left.result_id, right.result_id);
        assert_eq!(left.input, right.input);
        assert_eq!(left.status, right.status);
        assert_eq!(
            left.completed_classical_steps,
            right.completed_classical_steps
        );
        assert_eq!(left.observed_peak, right.observed_peak);
    }
}

#[test]
fn exp001_reproduces_independent_fixed_counts_and_peaks() {
    let configuration_path = runnable_fixture("experiments/EXP-001.json");
    let run = run_configuration(configuration_path.path()).expect("EXP-001 succeeds");
    let expected = HashMap::from([
        ("literal-1", (0, "1")),
        ("literal-2", (1, "2")),
        ("literal-3", (7, "16")),
        ("literal-27", (111, "9232")),
    ]);

    assert_eq!(run.records.len(), 4);
    for record in run.records {
        let (steps, peak) = expected[record.input.definition.input_id.as_str()];
        assert_eq!(record.status, ExperimentStatus::ReachedOne);
        assert_eq!(record.classical_steps_to_one.value, Some(steps));
        assert_eq!(
            record.classical_steps_to_one.completeness,
            MetricCompleteness::Complete
        );
        assert_eq!(record.observed_peak.value.as_deref(), Some(peak));
        assert_eq!(
            record.observed_peak.completeness,
            MetricCompleteness::Complete
        );
        assert_eq!(record.validation_state, ValidationState::Validated);
        if record.input.definition.input_id == "literal-1" {
            assert_eq!(
                record.first_descent.completeness,
                MetricCompleteness::Unavailable
            );
            assert_eq!(record.first_descent.value, None);
        }
    }
}

#[test]
fn all_supported_catalog_definitions_execute_consistently_across_three_engines() {
    let runs = [
        EnginePolicy::Reference,
        EnginePolicy::Bigint,
        EnginePolicy::Hybrid,
    ]
    .map(|policy| {
        let path = write_configuration(&all_catalog_configuration(policy));
        run_configuration(path.path()).expect("catalog run succeeds")
    });

    assert!(runs.iter().all(|run| run.records.len() == 10));
    for index in 0..10 {
        let reference = &runs[0].records[index];
        for run in &runs[1..] {
            let compared = &run.records[index];
            assert_eq!(
                compared.input.definition.input_id,
                reference.input.definition.input_id
            );
            assert_eq!(compared.status, reference.status);
            assert_eq!(
                compared.completed_classical_steps,
                reference.completed_classical_steps
            );
            assert_eq!(
                compared.classical_steps_to_one,
                reference.classical_steps_to_one
            );
            assert_eq!(compared.observed_peak, reference.observed_peak);
            assert_eq!(compared.first_descent, reference.first_descent);
            assert_eq!(compared.last_value, reference.last_value);
        }
    }
}

#[test]
fn incomplete_runs_label_prefix_metrics_without_inventing_steps_to_one() {
    let mut configuration = load_configuration("experiments/EXP-001.json");
    configuration.catalog_path = repository_path("catalog/inputs-v1.jsonl")
        .to_string_lossy()
        .into_owned();
    configuration.input_ids = vec!["literal-27".into()];
    configuration.limits.classical_step_limit = 1;
    let path = write_configuration(&configuration);
    let run = run_configuration(path.path()).expect("limited run succeeds");
    let result = &run.records[0];

    assert_eq!(result.status, ExperimentStatus::StepLimitReached);
    assert_eq!(result.completed_classical_steps, 1);
    assert_eq!(
        result.classical_steps_to_one.completeness,
        MetricCompleteness::Unavailable
    );
    assert_eq!(result.classical_steps_to_one.value, None);
    assert_eq!(
        result.observed_peak.completeness,
        MetricCompleteness::Prefix
    );
    assert_eq!(result.observed_peak.value.as_deref(), Some("82"));
    assert_eq!(
        result.first_descent.completeness,
        MetricCompleteness::Prefix
    );
}

#[test]
fn reference_overflow_is_distinct_and_counts_no_failed_transition() {
    let catalog_path = temp_path("overflow-catalog.jsonl");
    let definition = NumberDefinition {
        schema_version: 1,
        input_id: "u128-max".into(),
        name: "u128 maximum".into(),
        family: "boundary".into(),
        construction: NumberConstruction::Literal {
            value: u128::MAX.to_string(),
        },
        provenance: Provenance {
            origin: ValueOrigin::Generated,
            source_kind: ProvenanceSource::Local,
            source: "Rust u128 boundary".into(),
            external_id: None,
            retrieval_date: None,
            imported_value_sha256: None,
            reconstruction_note: "Reconstruct as 2^128 - 1.".into(),
        },
        declared_bit_length: 128,
        declared_decimal_digits: 39,
    };
    let line = serde_json::to_string(&definition).expect("boundary definition serializes");
    std::fs::write(catalog_path.path(), format!("{line}\n")).expect("boundary catalog writes");

    let mut configuration = load_configuration("experiments/EXP-001.json");
    configuration.catalog_path = catalog_path.path().to_string_lossy().into_owned();
    configuration.input_ids = vec!["u128-max".into()];
    configuration.limits.classical_step_limit = 1;
    let configuration_path = write_configuration(&configuration);
    let run = run_configuration(configuration_path.path()).expect("overflow is recorded as data");
    let result = &run.records[0];

    assert_eq!(result.status, ExperimentStatus::EngineError);
    assert_eq!(
        result.engine_outcome,
        EngineOutcome::ReferenceArithmeticOverflow
    );
    assert_eq!(result.completed_classical_steps, 0);
    assert_eq!(
        result.observed_peak.completeness,
        MetricCompleteness::Prefix
    );
    assert_eq!(
        result.last_value.as_deref(),
        Some(u128::MAX.to_string().as_str())
    );
    assert_eq!(result.validation_state, ValidationState::Validated);
}

#[test]
fn reference_input_above_u128_is_a_validated_engine_error() {
    let catalog_path = temp_path("above-u128-catalog.jsonl");
    let definition = NumberDefinition {
        schema_version: 1,
        input_id: "above-u128".into(),
        name: "One above u128 maximum".into(),
        family: "boundary".into(),
        construction: NumberConstruction::Literal {
            value: "340282366920938463463374607431768211456".into(),
        },
        provenance: Provenance {
            origin: ValueOrigin::Generated,
            source_kind: ProvenanceSource::Local,
            source: "Rust u128 boundary".into(),
            external_id: None,
            retrieval_date: None,
            imported_value_sha256: None,
            reconstruction_note: "Reconstruct exactly as 2^128.".into(),
        },
        declared_bit_length: 129,
        declared_decimal_digits: 39,
    };
    let line = serde_json::to_string(&definition).expect("boundary definition serializes");
    std::fs::write(catalog_path.path(), format!("{line}\n")).expect("boundary catalog writes");

    let mut configuration = load_configuration("experiments/EXP-001.json");
    configuration.catalog_path = catalog_path.path().to_string_lossy().into_owned();
    configuration.input_ids = vec!["above-u128".into()];
    let configuration_path = write_configuration(&configuration);
    let run = run_configuration(configuration_path.path()).expect("engine error is recorded");
    let result = &run.records[0];

    assert_eq!(result.status, ExperimentStatus::EngineError);
    assert_eq!(result.engine_outcome, EngineOutcome::InputNotRepresentable);
    assert_eq!(result.completed_classical_steps, 0);
    assert_eq!(
        result.observed_peak,
        collatz_experiments::LabeledMetric::unavailable()
    );
    assert_eq!(result.last_value, None);
    assert_eq!(result.validation_state, ValidationState::Validated);
}

#[test]
fn invalid_catalog_definition_stops_before_any_result_is_written() {
    let catalog_path = temp_path("invalid-catalog.jsonl");
    let mut definition = reviewed_catalog()
        .get("mersenne-5")
        .expect("fixture exists")
        .definition()
        .clone();
    definition.construction = NumberConstruction::Mersenne { exponent: 0 };
    let line = serde_json::to_string(&definition).expect("invalid fixture serializes");
    std::fs::write(catalog_path.path(), format!("{line}\n")).expect("invalid catalog writes");

    let mut configuration = load_configuration("experiments/EXP-001.json");
    configuration.catalog_path = catalog_path.path().to_string_lossy().into_owned();
    configuration.input_ids = vec!["mersenne-5".into()];
    let configuration_path = write_configuration(&configuration);
    let error = run_configuration(configuration_path.path()).expect_err("invalid input must stop");
    assert_eq!(error.status_code(), "invalid_input");
}

#[test]
fn exceptional_state_is_serialized_conservatively() {
    let configuration_path = runnable_fixture("experiments/EXP-001.json");
    let mut record = run_configuration(configuration_path.path())
        .expect("fixture run succeeds")
        .records
        .remove(0);
    record.mark_needs_reproduction();
    let json = serde_json::to_value(record).expect("result serializes");
    assert_eq!(json["validation_state"], "needs_reproduction");
    assert_ne!(json["validation_state"], "reproduced");
}

proptest! {
    #[test]
    fn generated_controls_preserve_requested_bit_length(exponent in 4_u32..32) {
        let mersenne_value: rug::Integer = (rug::Integer::from(1) << exponent) - 1;
        let definition = NumberDefinition {
            schema_version: 1,
            input_id: format!("mersenne-{exponent}"),
            name: format!("Mersenne M{exponent}"),
            family: "mersenne".into(),
            construction: NumberConstruction::Mersenne { exponent },
            provenance: Provenance {
                origin: ValueOrigin::Generated,
                source_kind: ProvenanceSource::Local,
                source: "property fixture".into(),
                external_id: None,
                retrieval_date: None,
                imported_value_sha256: None,
                reconstruction_note: "Property-test reconstruction.".into(),
            },
            declared_bit_length: exponent,
            declared_decimal_digits: u32::try_from(mersenne_value.to_string().len())
                .expect("small property fixture digit count fits u32"),
        };
        let special = ValidatedNumber::validate(definition).expect("property fixture validates");
        let specification = ControlSpecification {
            algorithm: "chacha20".into(),
            algorithm_version: "rand_chacha-0.10.0".into(),
            seed_hex: "11".repeat(32),
            samples_per_input: 2,
            mapping_version: "sha256-subseed-little-endian-mask-v1".into(),
            rejection_policy: RejectionPolicy::mvp_default(),
        };
        let controls = generate_controls("property-controls", &special, &specification)
            .expect("control generation succeeds");

        prop_assert_eq!(controls.len(), 2);
        for control in controls {
            prop_assert_eq!(control.number.definition().declared_bit_length, exponent);
            prop_assert_ne!(control.number.decimal_value(), special.decimal_value());
        }
    }
}
