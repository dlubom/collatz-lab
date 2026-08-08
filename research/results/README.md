# Research Results

This directory stores small, reviewable result summaries. It is not a database
and does not hold enormous integer expansions or complete large trajectories.

## Storage rules

- Small text results may be versioned in Git when their provenance and format
  are reviewable.
- Full huge integers and full large trajectories remain outside Git.
- Every external large artifact is represented by its filename or stable URI,
  byte size, media/format type, and SHA-256 hash.
- Every result names its experiment ID, configuration ID, run ID, input ID,
  build-embedded program-source SHA-256, and executable-source dirty state.
- Every result states its engine policy, termination status, limits, and
  validation method.
- An unusual result is labeled `needs-reproduction` until independent
  confirmation completes; it is not called a record or discovery before then.

Minimum human-readable fields are defined in
[`docs/experimental-methodology.md`](../../docs/experimental-methodology.md).
Invalidated results are retained with a reason or moved to an archival location
that preserves their Git history; they are not silently deleted.

## MVP summary fields

Human-readable versioned summaries track only:

- experiment ID;
- number name and provenance;
- exact formula or generator parameters;
- input bit length;
- classical steps to `1` when known;
- peak value or peak bit length;
- first descent below the start when observed;
- termination status;
- elapsed execution time;
- program version or commit;
- validation method.

The machine-readable result contract may carry additional fields required for
exact reproduction, but the first logbook and Git summary format remains small.
