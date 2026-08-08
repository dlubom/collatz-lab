use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use sha2::{Digest, Sha256};

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

    let source_sha256 = program_source_sha256(&repository_root)?;

    let mut status_arguments = vec!["status", "--porcelain=v1", "--untracked-files=normal", "--"];
    status_arguments.extend_from_slice(PROGRAM_PATHS);
    let source_dirty = !run_git(&repository_root, &status_arguments)?.is_empty();

    println!("cargo:rustc-env=COLLATZ_PROGRAM_SOURCE_SHA256={source_sha256}");
    println!("cargo:rustc-env=COLLATZ_PROGRAM_SOURCE_DIRTY={source_dirty}");
    Ok(())
}

fn program_source_sha256(repository_root: &Path) -> Result<String, String> {
    let mut files = Vec::new();
    for relative_path in PROGRAM_PATHS {
        collect_program_files(repository_root, Path::new(relative_path), &mut files)?;
    }
    files.sort();
    files.dedup();

    let mut hasher = Sha256::new();
    hasher.update(b"collatz-lab-program-source-v1\0");
    for relative_path in files {
        let normalized_path = relative_path
            .to_str()
            .ok_or_else(|| {
                format!(
                    "program source path is not UTF-8: {}",
                    relative_path.display()
                )
            })?
            .replace('\\', "/");
        let bytes = fs::read(repository_root.join(&relative_path)).map_err(|error| {
            format!(
                "cannot read program source {}: {error}",
                relative_path.display()
            )
        })?;
        hasher.update(
            u64::try_from(normalized_path.len())
                .map_err(|_| "program source path is too long".to_owned())?
                .to_be_bytes(),
        );
        hasher.update(normalized_path.as_bytes());
        hasher.update(
            u64::try_from(bytes.len())
                .map_err(|_| "program source file is too large".to_owned())?
                .to_be_bytes(),
        );
        hasher.update(bytes);
    }

    let digest = hasher.finalize();
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    Ok(output)
}

fn collect_program_files(
    repository_root: &Path,
    relative_path: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let path = repository_root.join(relative_path);
    let metadata = fs::symlink_metadata(&path)
        .map_err(|error| format!("cannot inspect program source {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() {
        return Err(format!(
            "program source path must not be a symbolic link: {}",
            relative_path.display()
        ));
    }
    if metadata.is_file() {
        files.push(relative_path.to_path_buf());
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(format!(
            "program source path is neither a file nor a directory: {}",
            relative_path.display()
        ));
    }

    let mut children = fs::read_dir(&path)
        .map_err(|error| format!("cannot list program source {}: {error}", path.display()))?
        .map(|entry| {
            entry
                .map(|entry| relative_path.join(entry.file_name()))
                .map_err(|error| format!("cannot list program source {}: {error}", path.display()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    children.sort();
    for child in children {
        collect_program_files(repository_root, &child, files)?;
    }
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
    let packed_refs = git_directory.join("packed-refs");
    if packed_refs.exists() {
        println!("cargo:rerun-if-changed={}", packed_refs.display());
    }

    let symbolic_ref = run_git_optional(repository_root, &["symbolic-ref", "-q", "HEAD"])?;
    if let Some(symbolic_ref) = symbolic_ref {
        let ref_path = run_git(repository_root, &["rev-parse", "--git-path", &symbolic_ref])?;
        let ref_path = PathBuf::from(ref_path);
        let ref_path = if ref_path.is_absolute() {
            ref_path
        } else {
            repository_root.join(ref_path)
        };
        if ref_path.exists() {
            println!("cargo:rerun-if-changed={}", ref_path.display());
        }
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
