import Mathlib.Data.Nat.Basic
import Lean.Elab.Tactic.Omega

namespace Collatz

/-- The standard, unaccelerated Collatz map. Its mathematical domain in this
project is the positive naturals; the total definition on `Nat` keeps the
positivity premise visible in every theorem that uses that domain. -/
def classicalStep (n : ℕ) : ℕ :=
  if n % 2 = 0 then n / 2 else 3 * n + 1

theorem mod_two_eq_zero_or_one (n : ℕ) : n % 2 = 0 ∨ n % 2 = 1 := by
  omega

theorem parity_branches_exclusive (n : ℕ) : ¬(n % 2 = 0 ∧ n % 2 = 1) := by
  omega

@[simp]
theorem classicalStep_of_even {n : ℕ} (hn : n % 2 = 0) : classicalStep n = n / 2 := by
  simp [classicalStep, hn]

@[simp]
theorem classicalStep_of_odd {n : ℕ} (hn : n % 2 = 1) : classicalStep n = 3 * n + 1 := by
  simp [classicalStep, show n % 2 ≠ 0 by omega]

theorem three_mul_add_one_pos {n : ℕ} (hn : 0 < n) : 0 < 3 * n + 1 := by
  omega

theorem three_mul_add_one_even {n : ℕ} (hn : n % 2 = 1) : (3 * n + 1) % 2 = 0 := by
  omega

theorem classicalStep_pos {n : ℕ} (hn : 0 < n) : 0 < classicalStep n := by
  rcases mod_two_eq_zero_or_one n with heven | hodd
  · rw [classicalStep_of_even heven]
    omega
  · rw [classicalStep_of_odd hodd]
    exact three_mul_add_one_pos hn

end Collatz
