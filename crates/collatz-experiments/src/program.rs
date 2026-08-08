/// Commit containing the exact executable-source snapshot embedded at build time.
pub fn program_commit() -> &'static str {
    env!("COLLATZ_PROGRAM_COMMIT")
}

/// Whether executable-source paths differed from the embedded commit at build time.
pub fn program_source_dirty() -> bool {
    env!("COLLATZ_PROGRAM_SOURCE_DIRTY") == "true"
}
