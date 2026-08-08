# Benchmarking

- **Status:** Accepted methodology; PBI-003 baseline harness measured locally
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

- program-source SHA-256 and dirty-worktree state;
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

## PBI-003 local smoke measurement

The first short measurement ran on 2026-08-02 from commit `a164723` with a
clean tracked worktree before Criterion wrote ignored raw artifacts. This is
build and harness evidence, not a performance conclusion or comparison
baseline.

- Host: Mac mini with Apple M4, 10 cores, 16 GB memory; macOS 26.6; AC power.
- Isolation: no dedicated thermal stabilization or competing-load isolation;
  results must not be used for a performance claim.
- Toolchain: Rust/Cargo 1.97.1, `rug` 1.30.0,
  `gmp-mpfr-sys` 1.7.1 with GMP 6.3.0, Criterion 0.8.2.
- Profile: Cargo `bench` optimized profile, with no project-specific compiler
  flags.
- Sampling: 250 ms warm-up, 10 samples, 500 ms requested measurement time,
  Criterion's default outlier policy.
- Timed region: engine execution and its internal state allocation are timed;
  literal construction, BigInt input cloning for batched cases, parsing,
  serialization, and external I/O are outside the timed region. Hybrid
  promotion and its allocation are inside the promotion case.

Observed Criterion estimate intervals:

| Case | Input and limit | Estimate interval |
|---|---|---:|
| Reference even step | `6 -> 3` | 1.2113–1.2217 ns |
| Reference odd step | `3 -> 10` | 1.4221–1.4322 ns |
| Reference | `27`, 111 classical transitions | 106.57–106.99 ns |
| BigInt | `27`, 111 classical transitions | 4.3984–4.4594 us |
| Hybrid, no promotion | `27`, 111 classical transitions | 679.75–685.03 ns |
| Hybrid promotion | `u128::MAX`, 1 classical transition | 124.45–125.44 ns |
| BigInt special form | `2^256 - 1`, 64 classical transitions | 3.1396–3.2022 us |

Raw Criterion output remains under ignored build directories and is not
versioned as research evidence.

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
