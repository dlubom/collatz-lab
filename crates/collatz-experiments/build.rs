use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const PROGRAM_PATHS: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "crates/collatz-engine/Cargo.toml",
    "crates/collatz-engine/src",
    "crates/collatz-experiments/Cargo.toml",
    "crates/collatz-experiments/build.rs",
    "crates/collatz-experiments/src",
    "crates/collatz-cli/Cargo.toml",
    "crates/collatz-cli/src",
];

fn main() {
    if let Err(error) = emit_program_provenance() {
        eprintln!("cannot determine Collatz Lab program provenance: {error}");
        std::process::exit(1);
    }
}

fn emit_program_provenance() -> Result<(), String> {
    let manifest_directory = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| "CARGO_MANIFEST_DIR is unavailable".to_owned())?;
    let repository_root = manifest_directory
        .join("../..")
        .canonicalize()
        .map_err(|error| format!("cannot resolve repository root: {error}"))?;

    for path in PROGRAM_PATHS {
        println!(
            "cargo:rerun-if-changed={}",
            repository_root.join(path).display()
        );
    }
    emit_git_rerun_paths(&repository_root)?;

    let mut commit_arguments = vec!["log", "-1", "--format=%H", "--"];
    commit_arguments.extend_from_slice(PROGRAM_PATHS);
    let commit = run_git(&repository_root, &commit_arguments)?;
    if commit.len() != 40
        || !commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!("git returned invalid program commit {commit}"));
    }

    let mut status_arguments = vec!["status", "--porcelain=v1", "--untracked-files=normal", "--"];
    status_arguments.extend_from_slice(PROGRAM_PATHS);
    let source_dirty = !run_git(&repository_root, &status_arguments)?.is_empty();

    println!("cargo:rustc-env=COLLATZ_PROGRAM_COMMIT={commit}");
    println!("cargo:rustc-env=COLLATZ_PROGRAM_SOURCE_DIRTY={source_dirty}");
    Ok(())
}

fn emit_git_rerun_paths(repository_root: &Path) -> Result<(), String> {
    let git_directory = PathBuf::from(run_git(
        repository_root,
        &["rev-parse", "--absolute-git-dir"],
    )?);
    println!(
        "cargo:rerun-if-changed={}",
        git_directory.join("HEAD").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        git_directory.join("packed-refs").display()
    );

    let symbolic_ref = run_git_optional(repository_root, &["symbolic-ref", "-q", "HEAD"])?;
    if let Some(symbolic_ref) = symbolic_ref {
        let ref_path = run_git(repository_root, &["rev-parse", "--git-path", &symbolic_ref])?;
        let ref_path = PathBuf::from(ref_path);
        let ref_path = if ref_path.is_absolute() {
            ref_path
        } else {
            repository_root.join(ref_path)
        };
        println!("cargo:rerun-if-changed={}", ref_path.display());
    }
    Ok(())
}

fn run_git(repository_root: &Path, arguments: &[&str]) -> Result<String, String> {
    let output = git_output(repository_root, arguments)?;
    if !output.status.success() {
        return Err(format!(
            "git {} failed: {}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|error| format!("git output is not UTF-8: {error}"))
}

fn run_git_optional(repository_root: &Path, arguments: &[&str]) -> Result<Option<String>, String> {
    let output = git_output(repository_root, arguments)?;
    if !output.status.success() {
        return Ok(None);
    }
    String::from_utf8(output.stdout)
        .map(|value| Some(value.trim().to_owned()))
        .map_err(|error| format!("git output is not UTF-8: {error}"))
}

fn git_output(repository_root: &Path, arguments: &[&str]) -> Result<Output, String> {
    Command::new("git")
        .args(arguments)
        .current_dir(repository_root)
        .output()
        .map_err(|error| format!("cannot execute git: {error}"))
}
