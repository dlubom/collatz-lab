# Benchmarking

- **Status:** Accepted methodology; no benchmark has been run
- **Supported baseline:** scalar CPU execution on macOS Apple Silicon

Benchmarking begins only after the relevant Lean build, fixed examples,
properties, differential tests, and quality gates pass. A faster implementation
that changes values, step accounting, peaks, or termination semantics is a
defect, not an optimization.

## Questions benchmarks may answer

- What is the cost per bounded trajectory for declared input families?
- Where does the checked `u128` engine spend time?
- What overhead is introduced by `rug::Integer` and by promotion?
- Which input shapes trigger materially different allocation or arithmetic
  behavior?

The MVP sets no throughput target before measurement. Timing does not strengthen
a mathematical claim and is never used as an acceptance oracle.

## Required benchmark context

Every reported comparison records:

- program commit and dirty-worktree state;
- Rust, Cargo, `rug`, GMP, and Criterion versions;
- macOS version, Apple Silicon model, core count, and memory;
- power source/mode and relevant thermal or competing-load notes;
- build profile and compiler flags;
- benchmark case IDs, exact input construction, and step limits;
- warm-up, sample-size, measurement-time, and outlier policy;
- whether setup, parsing, allocation, promotion, or serialization is inside the
  timed region.

Criterion benchmark compilation is checked with:

```bash
cargo bench --workspace --no-run
```

Full measurement uses `cargo bench --workspace` only after correctness gates.

## Initial workloads

The first suite should separate:

1. a single classical step by branch;
2. bounded reference runs for small fixed inputs;
3. reference and BigInt runs over identical representable inputs;
4. a hybrid run that promotes before `u128` overflow;
5. declared special-number generators measured outside engine timing unless
   generator cost is the explicit subject.

Large trajectories are bounded by a declared step limit. Separate numeric
engines are reported independently; comparisons state that their representation
costs differ.

## Interpretation

Report distributions or Criterion estimates, not an isolated stopwatch value.
Treat changes smaller than measurement noise as inconclusive. Re-run surprising
results and retain raw Criterion artifacts outside Git when large, with hashes
and metadata in the research result record.

SIMD, GPU, parallel execution, and distributed execution remain outside the MVP.
They are reconsidered only after profiling identifies a bottleneck, correctness
evidence is complete, and a new ADR records the changed execution model.
