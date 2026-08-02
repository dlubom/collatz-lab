import Collatz.Accelerated
import Collatz.Generators
import Mathlib.Tactic.Positivity
import Mathlib.Tactic.Ring

namespace Collatz

/-- The `j`-th predictable accelerated checkpoint for `a * 2^m - 1`. -/
def affineCheckpoint (a m j : ℕ) : ℕ :=
  3 ^ j * a * 2 ^ (m - j) - 1

private theorem checkpoint_base_eq_two_mul {a m j : ℕ} (hj : j < m) :
    3 ^ j * a * 2 ^ (m - j) = 2 * (3 ^ j * a * 2 ^ (m - j - 1)) := by
  have hexponent : m - j = (m - j - 1) + 1 := by omega
  conv_lhs => rw [hexponent, pow_succ]
  ring

private theorem next_checkpoint_base_eq_three_mul {a m j : ℕ} :
    3 ^ (j + 1) * a * 2 ^ (m - (j + 1)) =
      3 * (3 ^ j * a * 2 ^ (m - j - 1)) := by
  have hexponent : m - (j + 1) = m - j - 1 := by omega
  rw [hexponent, pow_succ]
  ring

private theorem checkpoint_eq_two_mul_sub_one {a m j : ℕ} (hj : j < m) :
    affineCheckpoint a m j = 2 * (3 ^ j * a * 2 ^ (m - j - 1)) - 1 := by
  unfold affineCheckpoint
  rw [checkpoint_base_eq_two_mul hj]

private theorem next_checkpoint_eq_three_mul_sub_one {a m j : ℕ} :
    affineCheckpoint a m (j + 1) = 3 * (3 ^ j * a * 2 ^ (m - j - 1)) - 1 := by
  unfold affineCheckpoint
  rw [next_checkpoint_base_eq_three_mul]

theorem affineCheckpoint_pos {a m j : ℕ} (ha : 1 ≤ a) (hj : j < m) :
    0 < affineCheckpoint a m j := by
  rw [checkpoint_eq_two_mul_sub_one hj]
  have hx : 0 < 3 ^ j * a * 2 ^ (m - j - 1) := by positivity
  omega

theorem affineCheckpoint_odd {a m j : ℕ} (ha : 1 ≤ a) (hj : j < m) :
    affineCheckpoint a m j % 2 = 1 := by
  rw [checkpoint_eq_two_mul_sub_one hj]
  have hx : 0 < 3 ^ j * a * 2 ^ (m - j - 1) := by positivity
  omega

/-- The arithmetic identity underlying every predictable checkpoint. -/
theorem affine_classical_identity {a m j : ℕ} (ha : 1 ≤ a) (hm : 2 ≤ m)
    (hj : j ≤ m - 2) :
    3 * affineCheckpoint a m j + 1 = 2 * affineCheckpoint a m (j + 1) := by
  have hjm : j < m := by omega
  let x := 3 ^ j * a * 2 ^ (m - j - 1)
  have hx : 0 < x := by
    dsimp [x]
    positivity
  rw [checkpoint_eq_two_mul_sub_one hjm, next_checkpoint_eq_three_mul_sub_one]
  change 3 * (2 * x - 1) + 1 = 2 * (3 * x - 1)
  omega

/-- In the documented range the next checkpoint is positive and odd, so the
preceding identity has 2-adic valuation exactly one. -/
theorem affine_accelerated_identity {a m j : ℕ} (ha : 1 ≤ a) (hm : 2 ≤ m)
    (hj : j ≤ m - 2) :
    nuTwo (3 * affineCheckpoint a m j + 1) = 1 ∧
      acceleratedStep (affineCheckpoint a m j) = affineCheckpoint a m (j + 1) := by
  have hjm : j < m := by omega
  have hcurrent_pos : 0 < affineCheckpoint a m j := affineCheckpoint_pos ha hjm
  have hnext_pos : 0 < affineCheckpoint a m (j + 1) :=
    affineCheckpoint_pos ha (by omega)
  have hnext_odd : affineCheckpoint a m (j + 1) % 2 = 1 :=
    affineCheckpoint_odd ha (by omega)
  have hidentity := affine_classical_identity ha hm hj
  have hpair : Nat.maxPowDvdDiv 2 (3 * affineCheckpoint a m j + 1) =
      (1, affineCheckpoint a m (j + 1)) := by
    apply Nat.maxPowDvdDiv_of_pow_mul_eq
    · omega
    · simpa using hidentity.symm
    · rw [Nat.dvd_iff_mod_eq_zero, hnext_odd]
      decide
  constructor
  · simp [nuTwo, padicValNat, hpair]
  · simp [acceleratedStep, oddPart, Nat.divMaxPow, hpair]

theorem affine_accelerated_weight {a m j : ℕ} (ha : 1 ≤ a) (hm : 2 ≤ m)
    (hj : j ≤ m - 2) : acceleratedWeight (affineCheckpoint a m j) = 2 := by
  rw [acceleratedWeight, (affine_accelerated_identity ha hm hj).1]

theorem affine_checkpoints_strictly_increase {a m j : ℕ} (ha : 1 ≤ a) (hm : 2 ≤ m)
    (hj : j ≤ m - 2) : affineCheckpoint a m j < affineCheckpoint a m (j + 1) := by
  have hjm : j < m := by omega
  let x := 3 ^ j * a * 2 ^ (m - j - 1)
  have hx : 0 < x := by
    dsimp [x]
    positivity
  rw [checkpoint_eq_two_mul_sub_one hjm, next_checkpoint_eq_three_mul_sub_one]
  change 2 * x - 1 < 3 * x - 1
  omega

theorem affinePowerTwo_eq_checkpoint (a m : ℕ) :
    affinePowerTwo a m = affineCheckpoint a m 0 := by
  simp [affinePowerTwo, affineCheckpoint]

theorem mersenne_eq_affinePowerTwo (m : ℕ) : mersenne m = affinePowerTwo 1 m := by
  simp [mersenne, affinePowerTwo]

theorem mersenne_eq_checkpoint (m : ℕ) : mersenne m = affineCheckpoint 1 m 0 := by
  rw [mersenne_eq_affinePowerTwo, affinePowerTwo_eq_checkpoint]

/-- The Mersenne case is a direct corollary of the general affine theorem. -/
theorem mersenne_accelerated_corollary {m j : ℕ} (hm : 2 ≤ m) (hj : j ≤ m - 2) :
    acceleratedStep (affineCheckpoint 1 m j) = affineCheckpoint 1 m (j + 1) :=
  (affine_accelerated_identity (a := 1) (m := m) (j := j) (by decide) hm hj).2

theorem mersenne_first_accelerated {m : ℕ} (hm : 2 ≤ m) :
    acceleratedStep (mersenne m) = affineCheckpoint 1 m 1 := by
  rw [mersenne_eq_checkpoint]
  exact mersenne_accelerated_corollary hm (by omega)

end Collatz
