# Collatz Lab

Collatz Lab is a correctness-first laboratory for reproducible experiments on
the standard Collatz map. It is designed to study selected known, structured,
unusual, and very large integers rather than to scan enormous contiguous
intervals.

The research question is deliberately empirical: do families such as Mersenne
numbers, Fermat numbers, repunits, values of the form `a * 2^m - 1`, RSA
Challenge numbers, Carmichael numbers, or record holders from other iterative
processes behave differently from random controls of the same bit length?
Interesting outcomes may include unusually long trajectories, late first
descent, large peaks, within-family records, reproducible discrepancies with a
published data source, or candidates that deserve independent investigation.

Collatz Lab does not claim that a large input is interesting merely because it
is large. A finite run is not a proof of the Collatz conjecture, absence of a
counterexample in a tested set is not a proof, and every exceptional result
must be reproduced through an independent computation before it is treated as
credible.

The planned MVP uses:

- Rust for a checked `u128` reference engine and a `rug::Integer`/GMP
  arbitrary-precision engine;
- Lean 4 for the mathematical reference model and local equivalence proofs;
- JSONL or an equivalent line-oriented result format for reproducible analysis;
- scalar CPU execution on macOS Apple Silicon.

The accepted project baseline is currently in the documentation and design
phase. No Rust engine, Lean project, benchmark result, experiment result, or
Collatz record is claimed yet. Implementation has not started.

## Documentation map

- [Living specification](specs/collatz-engine/spec.md) — current MVP contract
  and acceptance scenarios.
- [Project vision](docs/project-vision.md) — research purpose, hypotheses, and
  interpretation boundaries.
- [Mathematical definitions](docs/mathematical-definitions.md) — authoritative
  notation, step semantics, metrics, and proof boundary.
- [Experimental methodology](docs/experimental-methodology.md) — controls,
  provenance, statuses, and reproduction procedure.
- [Architecture](ARCHITECTURE.md) — components and dependency direction.
- [Quality strategy](docs/quality-strategy.md) — minimum MVP gates.
- [Contribution workflow](CONTRIBUTING.md) — branch, verification, commit,
  push, review, and merge policy.
- [ADRs](docs/adrs/README.md) — proposed foundational decisions.
- [PBIs](tasks/) — small implementation deltas; none has been executed.
- [Research logbook](research/logbook.md) — chronological decisions and
  observations once research begins.
