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
    assert!(stderr.starts_with("io_error: cannot "), "{stderr}");
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
