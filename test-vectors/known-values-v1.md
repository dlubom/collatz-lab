# Known Collatz Values v1

These literal values are independent acceptance oracles for the checked Rust
reference engine. They are reviewed in
[`docs/mathematical-definitions.md`](../docs/mathematical-definitions.md#14-fixed-independent-examples)
and checked separately by the Lean theorems in
[`lean/Collatz/TestVectors.lean`](../lean/Collatz/TestVectors.lean).

| Start | Classical steps through first `1` | Classical peak |
|---:|---:|---:|
| 1 | 0 | 1 |
| 2 | 1 | 2 |
| 3 | 7 | 16 |
| 27 | 111 | 9232 |

The Rust tests store these as literals. They do not call the engine to
synthesize an expected sequence, count, or peak. The full trajectory for `27`
is intentionally not copied into this file.

The initial generator oracle is `M_5 = 2^5 - 1 = 31`, checked by Lean theorem
`vector_mersenne_five`.
