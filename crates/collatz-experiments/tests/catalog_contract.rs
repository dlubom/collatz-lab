use std::collections::HashMap;
use std::path::PathBuf;

use collatz_experiments::{
    Catalog, CatalogError, NumberConstruction, NumberDefinition, NumberValidationError, Provenance,
    ValidatedNumber, ValueOrigin,
};

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
    assert!(matches!(
        ValidatedNumber::validate(definition),
        Err(NumberValidationError::ImportedSha256Mismatch { .. })
    ));
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
    assert!(matches!(
        ValidatedNumber::validate(mismatched),
        Err(NumberValidationError::MetadataMismatch {
            field: "declared_bit_length",
            declared: 6,
            actual: 5
        })
    ));

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
fn all_three_version_one_schemas_are_reviewable_json_documents() {
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
