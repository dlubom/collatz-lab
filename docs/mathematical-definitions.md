# Mathematical Definitions

- **Status:** Proposed authority for the Collatz engine
- **Scope:** Standard, unaccelerated Collatz dynamics over positive integers

This document fixes the mathematical language used by the Spec, Lean4 model,
Rust engines, tests, and benchmarks. It separates mathematical claims from
finite computational observations.

## Domain and notation

Let

\[
\mathbb{N}_{>0} = \{1,2,3,\ldots\}.
\]

Engine inputs belong to \(\mathbb{N}_{>0}\). Zero is rejected before a
transition is attempted.

For integers \(a\) and \(b>0\), \(a \bmod b\) denotes the least non-negative
remainder. Thus `n` is even when \(n \bmod 2=0\), and odd when
\(n \bmod 2=1\).

## The standard Collatz map

The project uses the unaccelerated map

\[
C(n)=
\begin{cases}
\dfrac{n}{2}, & n \equiv 0 \pmod 2,\\[6pt]
3n+1, & n \equiv 1 \pmod 2.
\end{cases}
\]

One application of either branch is one **transition**. In particular, the odd
step is exactly \(3n+1\); divisions by two that may follow are separate
transitions.

The mathematical map has \(C(1)=4\). The bounded engine runner nevertheless
halts when it first *observes* `1`, before applying another map step. The map
and the runner's terminal policy are distinct definitions.

## Iterates and trajectories

Define the iterates recursively:

\[
C^0(n)=n, \qquad C^{k+1}(n)=C(C^k(n)).
\]

For start \(n\), its trajectory is the sequence \((x_k)_{k\ge 0}\) with

\[
x_0=n, \qquad x_{k+1}=C(x_k).
\]

The starting value has index zero. A list containing values from \(x_0\)
through \(x_k\) therefore contains \(k+1\) values and represents exactly
\(k\) completed transitions.

## Finite run semantics

Every public engine run receives a transition limit \(L\in\mathbb{N}\).
For an ideal exact-arithmetic trajectory, define

\[
K(n,L)=\min\bigl(\{k\in\{0,\ldots,L\}:x_k=1\}\bigr)
\]

when that set is non-empty.

- If \(K(n,L)\) exists, the run returns `ReachedOne` after \(K(n,L)\)
  transitions with last value \(1\).
- Otherwise, the exact-arithmetic run returns `StepLimitReached` after \(L\)
  transitions with last value \(x_L\).

This definition gives terminal observation priority over limit exhaustion:
`run(1, 0)` returns `ReachedOne`, while a non-terminal start with limit zero
returns `StepLimitReached` without applying \(C\).

For a represented prefix \(x_0,\ldots,x_k\), its peak is

\[
P_k(n)=\max_{0\le i\le k}x_i.
\]

The start is always included. On bounded or overflowed runs, “peak” means the
maximum of the successfully represented prefix, not an assertion about the
unobserved remainder of the mathematical trajectory.

## Stopping-time terminology

Literature uses “stopping time” inconsistently, so engine interfaces and reports
must use explicit names.

The **first-descent time**, when it exists, is

\[
\sigma(n)=\min\{k\ge 1:C^k(n)<n\}.
\]

The **total stopping time**, when it exists, is

\[
\sigma_\infty(n)=\min\{k\ge 0:C^k(n)=1\}.
\]

The bounded runner reports a completed transition count and termination reason.
It may label that count `total_stopping_time` only when termination is
`ReachedOne`. It makes no stopping-time claim when the limit is reached or
`u128` arithmetic overflows.

## Numeric representations

### Checked `u128`

Let

\[
M=2^{128}-1.
\]

The reference engine represents values in \(\{0,\ldots,M\}\), while accepting
only positive starts. The even branch is representable for every accepted
`u128` value. For odd \(n\), the next value is representable exactly when

\[
n\le \left\lfloor\frac{M-1}{3}\right\rfloor.
\]

If this condition fails, the engine reports `ArithmeticOverflow` at the current
represented value. It does not increment the transition count and does not add
an unrepresentable next value to the peak.

### `rug::Integer` and GMP

The arbitrary-precision engine interprets `rug::Integer` values as mathematical
integers and restricts starts to \(\mathbb{N}_{>0}\). Its Collatz arithmetic is
exact: it does not have a numeric overflow outcome. The finite transition limit
still bounds the number of executed steps.

## Cross-engine equivalence

For a positive start \(n\le M\) and limit \(L\), suppose every exact value in
the observed prefix through termination or \(L\) transitions is at most \(M\).
Then the two engines are required to produce equal:

- represented values at every index;
- completed transition count;
- peak;
- `ReachedOne` or `StepLimitReached` classification.

This is a conformance obligation tested over examples and generated inputs. It
does not make the arbitrary-precision engine dependent on the reference engine
at runtime.

## Lean4 proof obligations

Lean4 models \(C\) over natural numbers with an explicit positivity premise or
positive-number subtype. Before optimization work is accepted, the formal layer
must prove at least:

1. **Branch completeness:** every positive `n` is in exactly one parity branch.
2. **Positivity preservation:** \(C(n)>0\) whenever \(n>0\).
3. **Even-branch decrease:** if \(n>1\) is even, then \(C(n)<n\).
4. **Odd-result parity:** if \(n\) is odd, then \(3n+1\) is even.
5. **Odd-branch growth:** if \(n>0\) is odd, then \(3n+1>n\).
6. **Iteration accounting:** a prefix ending at \(C^k(n)\) contains \(k+1\)
   values and represents \(k\) transitions.
7. **Terminal convention:** a bounded run whose current value is `1` records no
   additional transition.

Additional proofs may be added as algorithms evolve. If an accelerated map is
ever proposed, it requires a new definition, explicit correspondence theorems,
updated scenarios, and human approval; it is not silently interchangeable with
\(C\).

Lean4 proves properties of the mathematical model. The initial architecture
does not extract Rust from Lean or formalize Rust/GMP operational semantics, so
equivalence between proofs and executable code is established through review,
fixed examples, property-based checks, and cross-engine differential tests.

## Fixed examples

These examples are acceptance oracles:

| Start | Values through first `1` | Transitions | Peak |
|---:|---|---:|---:|
| 1 | `1` | 0 | 1 |
| 2 | `2, 1` | 1 | 2 |
| 3 | `3, 10, 5, 16, 8, 4, 2, 1` | 7 | 16 |
| 6 | `6, 3, 10, 5, 16, 8, 4, 2, 1` | 8 | 16 |
| 27 | _full sequence omitted_ | 111 | 9232 |

The omitted sequence for `27` must be produced and checked by tests rather than
duplicated in this document.

## Explicit non-claims

- Finite verification, however extensive, is not a proof that every positive
  integer reaches one.
- A `StepLimitReached` result is an incomplete finite observation, not evidence
  of divergence.
- Benchmark speed does not strengthen a mathematical claim.
- The initial project makes no claim about accelerated Collatz variants,
  negative integers, rational extensions, or the inverse Collatz graph.
