import Collatz.Basic
import Mathlib.Algebra.Order.BigOperators.Group.Finset
import Mathlib.Algebra.Ring.GeomSum
import Mathlib.Data.Finset.Range
import Mathlib.Tactic.NormNum

namespace Collatz

open scoped BigOperators

def mersenne (p : ℕ) : ℕ := 2 ^ p - 1

def fermat (k : ℕ) : ℕ := 2 ^ (2 ^ k) + 1

def repunit (base digits : ℕ) : ℕ :=
  ∑ i ∈ Finset.range digits, base ^ i

def affinePowerTwo (a m : ℕ) : ℕ := a * 2 ^ m - 1

theorem mersenne_eq_formula (p : ℕ) : mersenne p = 2 ^ p - 1 := rfl

theorem fermat_eq_formula (k : ℕ) : fermat k = 2 ^ (2 ^ k) + 1 := rfl

theorem repunit_eq_sum (base digits : ℕ) :
    repunit base digits = ∑ i ∈ Finset.range digits, base ^ i := rfl

theorem affinePowerTwo_eq_formula (a m : ℕ) : affinePowerTwo a m = a * 2 ^ m - 1 := rfl

theorem mersenne_pos {p : ℕ} (hp : 1 ≤ p) : 0 < mersenne p := by
  have hpow : 1 < 2 ^ p := Nat.one_lt_pow (by omega) (by norm_num)
  simp only [mersenne]
  omega

theorem fermat_pos (k : ℕ) : 0 < fermat k := by
  simp [fermat]

theorem repunit_pos {base digits : ℕ} (_hbase : 2 ≤ base) (hdigits : 1 ≤ digits) :
    0 < repunit base digits := by
  apply Finset.sum_pos'
  · intro i _
    exact Nat.zero_le (base ^ i)
  · refine ⟨0, ?_, by simp⟩
    simp
    omega

/-- The sum definition agrees with the quotient formula on the documented
base domain. -/
theorem repunit_eq_quotient {base digits : ℕ} (hbase : 2 ≤ base) :
    repunit base digits = (base ^ digits - 1) / (base - 1) := by
  apply Nat.eq_div_of_mul_eq_right (by omega)
  simpa [repunit, Nat.mul_comm] using
    geom_sum_mul_of_one_le (show 1 ≤ base by omega) digits

theorem affinePowerTwo_pos {a m : ℕ} (ha : 1 ≤ a) (hm : 1 ≤ m) :
    0 < affinePowerTwo a m := by
  have hpow : 1 < 2 ^ m := Nat.one_lt_pow (by omega) (by norm_num)
  have hmul : 2 ^ m ≤ a * 2 ^ m := by
    simpa using Nat.mul_le_mul_right (2 ^ m) ha
  simp only [affinePowerTwo]
  omega

end Collatz
