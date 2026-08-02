# Mathematical Definitions

- **Status:** Accepted mathematical authority for Collatz Lab
- **Scope:** Standard Collatz dynamics over positive integers, finite execution,
  compression semantics, metrics, and MVP generators

This document fixes the language used by the living Spec, Lean 4 model, Rust
engines, tests, experiment records, and benchmarks. Mathematical definitions
are distinct from finite computational observations.

## 1. Domain and notation

Let

\[
\mathbb{N}_{>0}=\{1,2,3,\ldots\}.
\]

The program's mathematical input domain is `N>0`. Zero, negative values,
rationals, and inverse trajectories are outside the MVP. Zero is rejected
before any transition.

For `z > 0`, define the 2-adic valuation

\[
\nu_2(z)=\max\{r\in\mathbb{N}:2^r\mid z\}.
\]

Thus `nu_2(z) = 0` exactly when `z` is odd. Let

\[
\ell_2(z)=\lfloor\log_2 z\rfloor+1
\]

denote the positive integer's bit length.

## 2. Classical Collatz step

The project uses the standard, unaccelerated Collatz map

\[
C(n)=
\begin{cases}
n/2, & n\equiv 0\pmod 2,\\[4pt]
3n+1, & n\equiv 1\pmod 2.
\end{cases}
\]

One application of either branch is exactly one **classical transition**. In
particular, `3n + 1` and every later division by two are separate classical
transitions.

The mathematical function has `C(1) = 4`. An engine runner stops when it first
observes `1` and therefore does not apply `C` to the terminal value. Function
definition and runner stopping policy are separate concepts.

## 3. Iteration and trajectory

Define iterates recursively:

\[
C^0(n)=n,\qquad C^{k+1}(n)=C(C^k(n)).
\]

The classical trajectory from `n` is the sequence `(x_k)` where

\[
x_0=n,\qquad x_{k+1}=C(x_k).
\]

The start has index zero. A represented prefix `x_0, ..., x_k` contains `k+1`
values and exactly `k` completed classical transitions.

## 4. Reaching one and bounded execution

The trajectory **reaches one at index `k`** when `x_k = 1`. The first such index,
when it exists, is

\[
\tau(n)=\min\{k\in\mathbb{N}:C^k(n)=1\}.
\]

This project calls `tau(n)` the **classical steps to one** or **total stopping
time**. The term is reported only when the complete count is known.

Every execution receives a classical-transition limit `L >= 0`. For exact
arithmetic, let

\[
K(n,L)=\min\{k\in\{0,\ldots,L\}:x_k=1\}
\]

when this set is non-empty. If it exists, execution returns `reached_one` after
`K(n,L)` transitions. Otherwise it returns `step_limit_reached` at `x_L` after
exactly `L` transitions.

Terminal observation has priority over limit exhaustion: start `1` with limit
zero reaches one; any nonterminal positive start with limit zero is a
zero-transition limited prefix.

Time and resource limits are operational limits defined in
[`experimental-methodology.md`](experimental-methodology.md). They do not alter
the mathematical trajectory.

## 5. Accelerated odd step

For odd `n > 0`, `3n + 1` is positive and even. Define the **accelerated odd
step**

\[
A(n)=\frac{3n+1}{2^{\nu_2(3n+1)}}.
\]

`A(n)` is positive and odd. It corresponds to exactly

\[
w_A(n)=1+\nu_2(3n+1)
\]

classical transitions: one odd transition to `3n+1`, followed by all
`nu_2(3n+1)` consecutive divisions by two. `A` is defined only on positive odd
inputs; it is not the classical map and one application is not one classical
transition.

## 6. Compressed iteration

Define one **compressed iteration** `S` for a current positive nonterminal
checkpoint:

\[
S(n)=
\begin{cases}
n/2^{\nu_2(n)}, & n>1\text{ and }n\text{ is even},\\[6pt]
A(n), & n>1\text{ and }n\text{ is odd}.
\end{cases}
\]

Its classical weight is

\[
w_S(n)=
\begin{cases}
\nu_2(n), & n\text{ even},\\
1+\nu_2(3n+1), & n\text{ odd}.
\end{cases}
\]

For every `n > 1`,

\[
S(n)=C^{w_S(n)}(n).
\]

Compressed execution starts with `y_0=n`; while `y_j != 1`, it records weight
`w_S(y_j)` and sets `y_{j+1}=S(y_j)`. The checkpoints are a subsequence of the
classical trajectory. The **compressed iteration count** is the number of
applications of `S`, not the sum of their weights.

After `r` compressed iterations, the corresponding number of classical steps
is exactly

\[
W_r=\sum_{j=0}^{r-1}w_S(y_j),
\qquad y_r=C^{W_r}(n).
\]

A limit expressed in classical transitions must never be overshot by executing
an indivisible macro. An implementation either expands the final macro or
stops at the last checkpoint whose accumulated weight is at most the limit.

### Peak preservation under compression

Compressed checkpoints alone do not contain every classical value. For an even
macro, intermediate halvings decrease. For an odd macro, the first intermediate
value `3n+1` is at least every following halved value. Therefore exact classical
peak accounting under compression must consider `3n+1` before discarding powers
of two. Taking only `max(y_j)` may underreport the classical peak.

## 7. Multi-iteration jumps

A **multi-iteration jump** is any future formula or table lookup that replaces
two or more applications of `S` without explicitly visiting every compressed
checkpoint. It is distinct from the classical step, accelerated odd step, and
one compressed iteration. Such jumps are outside the MVP and require their own
equivalence theorem, step-weight proof, peak-accounting rule, and Spec update.

## 8. Trajectory metrics

### Classical steps to one

For a completed trajectory this is `tau(n)`. For an incomplete prefix it is
unavailable, not the number of transitions executed so far.

### Compressed iterations

For a completed compressed execution, define

\[
\kappa(n)=\min\{r\in\mathbb{N}:y_r=1\}
\]

when it exists. `kappa(n)` and `tau(n)` are different metrics; the relation is
the sum of classical weights.

### Maximum

For a completed trajectory,

\[
P(n)=\max_{0\le k\le\tau(n)}x_k.
\]

For a represented prefix through index `r`,

\[
P_r(n)=\max_{0\le k\le r}x_k.
\]

The start is included. A bounded or interrupted run reports `P_r(n)` as an
**observed prefix peak**, never as the unknown full-trajectory maximum.

The **peak ratio** is `P(n)/n` for a complete result, or explicitly
`P_r(n)/n` for a prefix. The **bit-length gain** is

\[
\ell_2(P)-\ell_2(n)
\]

using the corresponding complete or prefix peak.

### First descent below the start

The **first-descent time**, when it exists, is

\[
\sigma(n)=\min\{k\ge 1:C^k(n)<n\}.
\]

If no represented prefix value is below `n`, the bounded result records
first descent as not yet observed. For start `1`, a runner terminates before a
descent and the metric is unavailable.

### Initial compressed growth run

The **initial compressed growth-run length** is

\[
g(n)=\max\{r\ge0:y_{j+1}>y_j\text{ for all }0\le j<r\},
\]

with the value censored when execution stops before the first non-increase. This
metric concerns compressed checkpoints and must not be described as a run of
consecutive increasing classical steps.

### Runtime and engine promotions

Elapsed execution time is an operational measurement, not a mathematical
invariant. An **engine promotion** is a change in numeric representation during
one execution. The MVP hybrid policy permits at most one `u128`-to-BigInt
promotion and records whether it occurred.

## 9. Verified bound

A **verified bound record** consists of an inclusive integer `B >= 1` plus a
cited, versioned claim that every `z` in `[1,B]` reaches one. A run **reaches the
verified bound** when an executed checkpoint `x_k` satisfies `1 <= x_k <= B`
and the exact bound record is part of its configuration.

This may justify stopping further local computation, but it does not by itself
provide the suffix's classical step count or peak. Safe early termination must
record `x_k`, `B`, the source/version, and prefix metrics. It may compose total
metrics only when independently verified suffix data supplies the necessary
values and the composition is checked. A verified bound is finite evidence, not
a proof of universal Collatz termination.

## 10. Numeric representations

### Checked `u128`

Let

\[
M=2^{128}-1.
\]

For an odd represented value `n`, `3n+1` is representable exactly when

\[
n\le\left\lfloor\frac{M-1}{3}\right\rfloor.
\]

The bounded reference engine uses checked multiplication and addition. If the
condition fails, it reports arithmetic overflow at the current value, does not
increment the transition count, and does not invent a next value or peak.

### Arbitrary precision and promotion

The `rug::Integer` engine interprets positive values as mathematical integers
and performs exact Collatz arithmetic. The hybrid runner checks representability
before an odd transition. If the next classical value would exceed `M`, it
promotes the current represented value to BigInt and performs that same
transition exactly; the transition count increases once after the result exists.

Promotion does not change the trajectory, terminal policy, limits, or metrics.
The standalone bounded reference engine still reports overflow rather than
promoting, which preserves it as an independent bounded oracle.

### Cross-engine equivalence

For positive `n <= M` and a finite limit, if every value in the observed exact
prefix is at most `M`, the bounded and arbitrary-precision engines must agree on
every represented classical value, transition count, peak, first descent, and
termination classification. The hybrid runner must agree with the BigInt engine
through and after promotion.

## 11. Generators

Generators return positive integers together with their exact parameters. They
do not assert primality or special historical status unless separately sourced.

### Mersenne numbers

For integer `p >= 1`,

\[
M_p=2^p-1.
\]

The generator produces a Mersenne number, not necessarily a Mersenne prime.

### Fermat numbers

For integer `k >= 0`,

\[
F_k=2^{2^k}+1.
\]

The generator does not assert that `F_k` is prime.

### Repunits

For base `b >= 2` and length `d >= 1`,

\[
R_{b,d}=\sum_{i=0}^{d-1}b^i=\frac{b^d-1}{b-1}.
\]

This is the integer whose base-`b` representation contains `d` digits equal to
one.

### Values `a * 2^m - 1`

For integers `a >= 1` and `m >= 1`,

\[
N(a,m)=a2^m-1.
\]

It is positive and odd. Mersenne numbers are the subfamily `N(1,p)`.

### Later generators

Factorial neighbors `n! +/- 1`, primorial neighbors `p# +/- 1`, sourced
Carmichael numbers, pseudoprimes, RSA Challenge values, persistence record
holders, OEIS selections, and manual imports are deferred to later catalog
increments. Each requires a domain, parameter, provenance, and validation rule
before implementation.

## 12. Special-form identity

Let `a >= 1`, `m >= 2`, and

\[
n_j=3^j a2^{m-j}-1.
\]

For every `0 <= j <= m-2`, `n_j` is positive and odd, and

\[
3n_j+1=2\left(3^{j+1}a2^{m-j-1}-1\right).
\]

The parenthesized factor is odd, so `nu_2(3n_j+1)=1` and

\[
A(n_j)=n_{j+1}=3^{j+1}a2^{m-j-1}-1.
\]

Moreover,

\[
n_{j+1}-n_j=3^j a2^{m-j-1}>0.
\]

Thus `N(a,m)` has `m-1` predictable, strictly increasing accelerated
checkpoints before the terminal exponent-one case requires separate valuation
analysis. Setting `a=1` yields the Mersenne corollary. This is a local identity,
not a claim about later trajectory behavior or total stopping time.

## 13. Lean 4 obligations and boundary

The MVP formal model must establish:

1. parity branch exclusivity for positive naturals;
2. positivity preservation of `C`;
3. positivity and evenness of `3n+1` for odd positive `n`;
4. correspondence of `A(n)` with one odd step and exactly the required halvings;
5. compressed checkpoints `S(n)=C^(w_S(n))(n)`;
6. correctness of accumulated classical-step weights;
7. correct peak treatment of skipped intermediate values;
8. generator outputs equal their mathematical definitions;
9. the `a * 2^m - 1` identity and the Mersenne corollary;
10. small examples for `1`, `2`, `3`, and generator values.

Future mathematical optimizations require a theorem connecting them to this
reference model. Lean 4 does not attempt to prove the Collatz conjecture or the
compiled Rust implementation.

## 14. Fixed independent examples

These constants are acceptance oracles and must not be recomputed by the tested
Rust algorithm to create expected values:

| Start | Values through first `1` | Classical steps | Peak |
|---:|---|---:|---:|
| 1 | `1` | 0 | 1 |
| 2 | `2, 1` | 1 | 2 |
| 3 | `3, 10, 5, 16, 8, 4, 2, 1` | 7 | 16 |
| 27 | full sequence intentionally not duplicated here | 111 | 9232 |

Generator examples are `M_5=31`, `F_2=17`, `R_(10,3)=111`, and
`N(3,4)=47`.

## 15. Explicit non-claims

- No finite computation proves that every positive integer reaches one.
- No tested set without a counterexample proves the conjecture.
- A limit or resource stop is an incomplete observation, not evidence of
  divergence.
- A large peak or input is not important without a declared comparison.
- Benchmark speed is not correctness evidence.
- Lean's local theorems do not verify Rust, GMP, parsing, serialization, the
  operating system, or performance.
