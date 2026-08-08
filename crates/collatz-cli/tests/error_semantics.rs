use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static PATH_COUNTER: AtomicU64 = AtomicU64::new(0);

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn missing_output(suffix: &str) -> PathBuf {
    let counter = PATH_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir()
        .join(format!(
            "collatz-cli-{}-{counter}-missing",
            std::process::id()
        ))
        .join(suffix)
}

fn temporary_file(suffix: &str) -> PathBuf {
    let counter = PATH_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "collatz-cli-{}-{counter}-{suffix}",
        std::process::id()
    ))
}

fn run(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_collatz-cli"))
        .args(arguments)
        .current_dir(repository_root())
        .output()
        .expect("CLI process starts")
}

fn assert_io_error(output: Output) {
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    assert!(stderr.starts_with("io_error: "), "{stderr}");
    assert!(stderr.contains("cannot "), "{stderr}");
    assert!(!stderr.starts_with("invalid_input:"), "{stderr}");
}

#[test]
fn plan_and_run_output_failures_share_the_io_error_category() {
    for (action, configuration, suffix) in [
        ("plan", "experiments/EXP-002.json", "plan.json"),
        ("run", "experiments/EXP-001.json", "results.jsonl"),
    ] {
        let output_path = missing_output(suffix);
        assert!(!output_path.exists());
        let output = run(&[
            "experiment",
            action,
            configuration,
            "--output",
            output_path.to_str().expect("temporary path is UTF-8"),
        ]);
        assert_io_error(output);
    }
}

#[test]
fn missing_catalog_is_io_error_instead_of_invalid_input() {
    let path = missing_output("catalog.jsonl");
    assert!(!path.exists());
    assert_io_error(run(&[
        "catalog",
        "validate",
        path.to_str().expect("temporary path is UTF-8"),
    ]));
}

#[test]
fn missing_configuration_is_io_error_for_plan_and_run() {
    let configuration = missing_output("experiment.json");
    assert!(!configuration.exists());

    for action in ["plan", "run"] {
        let output = temporary_file(if action == "plan" {
            "plan.json"
        } else {
            "results.jsonl"
        });
        assert_io_error(run(&[
            "experiment",
            action,
            configuration.to_str().expect("temporary path is UTF-8"),
            "--output",
            output.to_str().expect("temporary path is UTF-8"),
        ]));
    }
}

#[test]
fn catalog_missing_through_configuration_is_io_error() {
    let catalog = missing_output("catalog.jsonl");
    let configuration = temporary_file("experiment.json");
    let output = temporary_file("plan.json");
    let source = std::fs::read_to_string(repository_root().join("experiments/EXP-001.json"))
        .expect("reviewed configuration can be read");
    let missing_catalog = catalog.to_str().expect("temporary path is UTF-8");
    let modified = source.replace(
        "\"catalog_path\": \"catalog/inputs-v1.jsonl\"",
        &format!("\"catalog_path\": \"{missing_catalog}\""),
    );
    assert_ne!(modified, source, "catalog path fixture must be replaced");
    std::fs::write(&configuration, modified).expect("temporary configuration can be written");

    assert_io_error(run(&[
        "experiment",
        "plan",
        configuration.to_str().expect("temporary path is UTF-8"),
        "--output",
        output.to_str().expect("temporary path is UTF-8"),
    ]));

    std::fs::remove_file(configuration).expect("temporary configuration can be removed");
}
