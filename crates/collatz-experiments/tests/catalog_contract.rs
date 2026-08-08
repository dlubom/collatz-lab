use std::collections::HashMap;
use std::path::PathBuf;

use collatz_experiments::{
    Catalog, CatalogError, ExperimentConfiguration, NumberConstruction, NumberDefinition,
    NumberValidationError, Provenance, ProvenanceSource, RunOutput, ValidatedNumber, ValueOrigin,
    run_configuration,
};
use serde_json::json;

fn repository_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn reviewed_catalog() -> Catalog {
    Catalog::load_jsonl(repository_path("catalog/inputs-v1.jsonl"))
        .expect("the reviewed catalog must validate")
}

#[test]
fn reviewed_catalog_contains_exactly_the_ten_authorized_reconstructions() {
    let catalog = reviewed_catalog();
    let actual: HashMap<_, _> = catalog
        .entries()
        .iter()
        .map(|entry| (entry.definition().input_id.as_str(), entry.decimal_value()))
        .collect();
    let expected = HashMap::from([
        ("literal-1", "1"),
        ("literal-2", "2"),
        ("literal-3", "3"),
        ("literal-27", "27"),
        ("mersenne-5", "31"),
        ("mersenne-7", "127"),
        ("mersenne-13", "8191"),
        ("fermat-2", "17"),
        ("repunit-10-3", "111"),
        ("affine-3-4", "47"),
    ]);

    assert_eq!(catalog.len(), 10);
    assert_eq!(actual.len(), expected.len());
    for (input_id, value) in expected {
        assert_eq!(actual.get(input_id).map(String::as_str), Some(value));
    }
}

#[test]
fn every_supported_constructor_enforces_its_documented_domain() {
    let catalog = reviewed_catalog();
    let cases = [
        (
            "literal-1",
            NumberConstruction::Literal { value: "0".into() },
        ),
        ("mersenne-5", NumberConstruction::Mersenne { exponent: 0 }),
        (
            "repunit-10-3",
            NumberConstruction::Repunit { base: 1, length: 3 },
        ),
        (
            "affine-3-4",
            NumberConstruction::AffinePowerOfTwo {
                coefficient: "0".into(),
                exponent: 4,
            },
        ),
    ];

    for (input_id, construction) in cases {
        let mut definition = catalog
            .get(input_id)
            .expect("fixture input exists")
            .definition()
            .clone();
        definition.construction = construction;
        let error = ValidatedNumber::validate(definition).expect_err("invalid domain must fail");
        assert_eq!(error.status_code(), "invalid_input");
    }
}

#[test]
fn fermat_zero_is_valid_and_the_shift_interface_boundary_is_explicit() {
    let catalog = reviewed_catalog();
    let mut definition = catalog
        .get("fermat-2")
        .expect("fixture input exists")
        .definition()
        .clone();
    definition.input_id = "fermat-0".into();
    definition.name = "Fermat F0".into();
    definition.construction = NumberConstruction::Fermat { index: 0 };
    definition.provenance.reconstruction_note = "Reconstruct as 2^(2^0) + 1.".into();
    definition.declared_bit_length = 2;
    definition.declared_decimal_digits = 1;

    assert_eq!(
        ValidatedNumber::validate(definition.clone())
            .expect("F0 is inside the documented domain")
            .decimal_value(),
        "3"
    );

    definition.construction = NumberConstruction::Fermat { index: 32 };
    assert!(matches!(
        ValidatedNumber::validate(definition),
        Err(NumberValidationError::ReconstructionLimit { .. })
    ));
}

#[test]
fn imported_literal_requires_complete_provenance_and_matching_sha256() {
    let catalog = reviewed_catalog();
    let mut definition = catalog
        .get("affine-3-4")
        .expect("fixture input exists")
        .definition()
        .clone();
    definition.input_id = "imported-47".into();
    definition.name = "Imported 47".into();
    definition.construction = NumberConstruction::Literal { value: "47".into() };
    definition.provenance = Provenance {
        origin: ValueOrigin::Imported,
        source_kind: ProvenanceSource::External,
        source: "reviewed external fixture".into(),
        external_id: Some("fixture-47".into()),
        retrieval_date: Some("2026-08-08".into()),
        imported_value_sha256: Some(
            "31489056e0916d59fe3add79e63f095af3ffb81604691f21cad442a85c7be617".into(),
        ),
        reconstruction_note: "Compare the imported decimal bytes with SHA-256.".into(),
    };

    assert_eq!(
        ValidatedNumber::validate(definition.clone())
            .expect("independently hashed fixture must validate")
            .decimal_value(),
        "47"
    );

    definition.provenance.imported_value_sha256 = Some("0".repeat(64));
    let error = ValidatedNumber::validate(definition)
        .expect_err("mismatched imported hash must fail verification");
    assert!(matches!(
        error,
        NumberValidationError::ImportedSha256Mismatch { .. }
    ));
    assert_eq!(error.status_code(), "verification_failed");
}

#[test]
fn derived_metadata_and_catalog_identifiers_are_not_repaired_silently() {
    let catalog = reviewed_catalog();
    let mut mismatched = catalog
        .get("mersenne-5")
        .expect("fixture input exists")
        .definition()
        .clone();
    mismatched.declared_bit_length = 6;
    let metadata_error =
        ValidatedNumber::validate(mismatched.clone()).expect_err("metadata drift must fail");
    assert!(matches!(
        metadata_error,
        NumberValidationError::MetadataMismatch {
            field: "declared_bit_length",
            declared: 6,
            actual: 5
        }
    ));
    assert_eq!(metadata_error.status_code(), "verification_failed");
    let catalog_error =
        Catalog::from_definitions(vec![mismatched]).expect_err("catalog must preserve status");
    assert_eq!(catalog_error.status_code(), "verification_failed");

    let duplicate = catalog
        .get("literal-1")
        .expect("fixture input exists")
        .definition()
        .clone();
    let error = Catalog::from_definitions(vec![duplicate.clone(), duplicate])
        .expect_err("duplicate stable IDs must fail");
    assert!(matches!(error, CatalogError::DuplicateInputId { .. }));
    assert_eq!(error.status_code(), "invalid_input");
}

#[test]
fn external_provenance_requires_a_retrieval_date_even_without_an_external_id() {
    let mut definition = reviewed_catalog()
        .get("literal-3")
        .expect("fixture exists")
        .definition()
        .clone();
    definition.provenance.source_kind = ProvenanceSource::External;

    let error = ValidatedNumber::validate(definition)
        .expect_err("every external source needs a retrieval date");
    assert!(matches!(
        error,
        NumberValidationError::RequiredProvenanceFieldMissing {
            field: "provenance.retrieval_date"
        }
    ));
    assert_eq!(error.status_code(), "invalid_input");
}

#[test]
fn invalid_utf8_in_a_catalog_is_invalid_input_not_an_io_failure() {
    let path =
        std::env::temp_dir().join(format!("collatz-invalid-utf8-{}.jsonl", std::process::id()));
    std::fs::write(&path, [0xff, b'\n']).expect("invalid UTF-8 fixture writes");
    let error = Catalog::load_jsonl(&path).expect_err("invalid UTF-8 must fail");
    let _ = std::fs::remove_file(path);

    assert!(matches!(error, CatalogError::InvalidEncoding { line: 1 }));
    assert_eq!(error.status_code(), "invalid_input");
}

#[test]
fn all_three_version_one_schemas_are_reviewable_json_documents() {
    let number_schema = schema("schemas/number-definition-v1.schema.json");
    let config_schema = schema("schemas/experiment-config-v1.schema.json");
    let result_schema = schema("schemas/result-v1.schema.json");
    let registry = jsonschema::Registry::new()
        .add(
            "https://github.com/dlubom/collatz-lab/schemas/number-definition-v1.schema.json",
            number_schema.clone(),
        )
        .expect("number schema ID is valid")
        .prepare()
        .expect("schema registry prepares");

    let number_validator = jsonschema::draft202012::options()
        .should_validate_formats(true)
        .with_registry(&registry)
        .build(&number_schema)
        .expect("number schema compiles as Draft 2020-12");
    let config_validator = jsonschema::draft202012::options()
        .should_validate_formats(true)
        .with_registry(&registry)
        .build(&config_schema)
        .expect("configuration schema compiles as Draft 2020-12");
    let result_validator = jsonschema::draft202012::options()
        .should_validate_formats(true)
        .with_registry(&registry)
        .build(&result_schema)
        .expect("result schema compiles as Draft 2020-12");

    for relative in [
        "schemas/number-definition-v1.schema.json",
        "schemas/experiment-config-v1.schema.json",
        "schemas/result-v1.schema.json",
    ] {
        let bytes = std::fs::read(repository_path(relative)).expect("schema file is readable");
        let schema: serde_json::Value =
            serde_json::from_slice(&bytes).expect("schema file is valid JSON");
        assert_eq!(
            schema["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert!(schema["$id"].as_str().is_some_and(|id| id.contains("v1")));
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["additionalProperties"], false);
    }

    let catalog = std::fs::read_to_string(repository_path("catalog/inputs-v1.jsonl"))
        .expect("catalog artifact reads");
    let number_instances: Vec<_> = catalog
        .lines()
        .map(|line| serde_json::from_str(line).expect("catalog line is JSON"))
        .collect();
    for instance in &number_instances {
        assert_schema_valid(&number_validator, instance);
    }

    for relative in ["experiments/EXP-001.json", "experiments/EXP-002.json"] {
        assert_schema_valid(&config_validator, &schema(relative));
    }

    let run = run_exp001();
    for record in run.records {
        let instance = serde_json::to_value(record).expect("result record serializes");
        assert_schema_valid(&result_validator, &instance);
    }

    let mut imported_missing_fields = number_instances[0].clone();
    imported_missing_fields["provenance"]["origin"] = json!("imported");
    imported_missing_fields["provenance"]["source_kind"] = json!("external");
    assert!(!number_validator.is_valid(&imported_missing_fields));

    let mut external_without_date = number_instances[0].clone();
    external_without_date["provenance"]["source_kind"] = json!("external");
    assert!(!number_validator.is_valid(&external_without_date));

    let mut fermat_outside_v1 = number_instances[7].clone();
    fermat_outside_v1["construction"]["index"] = json!(32);
    assert!(!number_validator.is_valid(&fermat_outside_v1));

    let mut unavailable_with_value =
        serde_json::to_value(run_exp001().records.remove(1)).expect("result record serializes");
    unavailable_with_value["classical_steps_to_one"] =
        json!({"completeness": "unavailable", "value": 1});
    assert!(!result_validator.is_valid(&unavailable_with_value));
}

fn schema(relative: &str) -> serde_json::Value {
    let bytes = std::fs::read(repository_path(relative)).expect("JSON artifact is readable");
    serde_json::from_slice(&bytes).expect("JSON artifact parses")
}

fn assert_schema_valid(validator: &jsonschema::Validator, instance: &serde_json::Value) {
    let errors: Vec<_> = validator
        .iter_errors(instance)
        .map(|error| error.to_string())
        .collect();
    assert!(errors.is_empty(), "schema errors: {errors:#?}");
}

fn run_exp001() -> RunOutput {
    let mut configuration =
        ExperimentConfiguration::load(repository_path("experiments/EXP-001.json"))
            .expect("EXP-001 configuration loads");
    configuration.catalog_path = repository_path("catalog/inputs-v1.jsonl")
        .to_string_lossy()
        .into_owned();
    let path =
        std::env::temp_dir().join(format!("collatz-schema-exp001-{}.json", std::process::id()));
    std::fs::write(
        &path,
        serde_json::to_vec_pretty(&configuration).expect("configuration serializes"),
    )
    .expect("temporary configuration writes");
    let run = run_configuration(&path).expect("EXP-001 produces result artifacts");
    let _ = std::fs::remove_file(path);
    run
}

#[test]
fn experiment_schema_pins_the_version_one_control_sample_maximum() {
    let bytes = std::fs::read(repository_path("schemas/experiment-config-v1.schema.json"))
        .expect("schema file is readable");
    let schema: serde_json::Value =
        serde_json::from_slice(&bytes).expect("schema file is valid JSON");

    assert_eq!(
        schema["$defs"]["controls"]["properties"]["samples_per_input"]["maximum"],
        4096
    );
    assert_eq!(schema["properties"]["input_ids"]["maxItems"], 16_384);
}

#[test]
fn serde_rejects_unknown_definition_fields() {
    let line = r#"{
        "schema_version":1,
        "input_id":"literal-1",
        "name":"One",
        "family":"manual",
        "construction":{"kind":"literal","value":"1"},
        "provenance":{
            "origin":"generated",
            "source_kind":"local",
            "source":"fixture",
            "external_id":null,
            "retrieval_date":null,
            "imported_value_sha256":null,
            "reconstruction_note":"fixture"
        },
        "declared_bit_length":1,
        "declared_decimal_digits":1,
        "unreviewed":true
    }"#;
    assert!(serde_json::from_str::<NumberDefinition>(line).is_err());
}
