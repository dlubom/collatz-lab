/// Stable SHA-256 of the exact executable-source snapshot embedded at build time.
pub fn program_source_sha256() -> &'static str {
    env!("COLLATZ_PROGRAM_SOURCE_SHA256")
}

/// Whether executable-source paths had Git worktree changes at build time.
pub fn program_source_dirty() -> bool {
    env!("COLLATZ_PROGRAM_SOURCE_DIRTY") == "true"
}
