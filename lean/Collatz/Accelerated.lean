import Collatz.Iteration

namespace Collatz

/-- The 2-adic valuation on nonzero naturals, with Mathlib's documented zero
convention outside the project's positive domain. -/
def nuTwo (n : ℕ) : ℕ := padicValNat 2 n

/-- Remove the maximal power of two from `n`. -/
def oddPart (n : ℕ) : ℕ := n.divMaxPow 2

theorem classicalStep_two_mul (n : ℕ) : classicalStep (2 * n) = n := by
  simp [classicalStep]

theorem iterate_pow_two_mul (k q : ℕ) : iterate k (2 ^ k * q) = q := by
  induction k with
  | zero => simp
  | succ k ih =>
      rw [iterate_succ]
      have hstep : classicalStep (2 ^ (k + 1) * q) = 2 ^ k * q := by
        simpa [pow_succ, Nat.mul_assoc, Nat.mul_comm, Nat.mul_left_comm] using
          classicalStep_two_mul (2 ^ k * q)
      rw [hstep]
      exact ih

/-- Dividing by two exactly `ν₂(n)` times reaches the odd part. -/
theorem iterate_nuTwo (n : ℕ) : iterate (nuTwo n) n = oddPart n := by
  calc
    iterate (nuTwo n) n =
        iterate (nuTwo n) (2 ^ padicValNat 2 n * n.divMaxPow 2) := by
          rw [Nat.pow_padicValNat_mul_divMaxPow]
    _ = oddPart n := by
      simpa [nuTwo, oddPart] using iterate_pow_two_mul (padicValNat 2 n) (n.divMaxPow 2)

theorem oddPart_pos {n : ℕ} (hn : 0 < n) : 0 < oddPart n := by
  apply Nat.pos_of_ne_zero
  intro hzero
  have hnzero : n = 0 := by
    rw [oddPart] at hzero
    have hproduct := Nat.divMaxPow_mul_pow_padicValNat 2 n
    rw [hzero, zero_mul] at hproduct
    exact hproduct.symm
  omega

theorem oddPart_odd {n : ℕ} (hn : 0 < n) : oddPart n % 2 = 1 := by
  have hnot : ¬2 ∣ oddPart n := by
    exact Nat.not_dvd_divMaxPow (by omega) hn.ne'
  rw [Nat.dvd_iff_mod_eq_zero] at hnot
  rcases mod_two_eq_zero_or_one (oddPart n) with hzero | hone
  · exact absurd hzero hnot
  · exact hone

/-- One accelerated odd step removes every factor of two from `3n + 1`. -/
def acceleratedStep (n : ℕ) : ℕ := oddPart (3 * n + 1)

/-- The exact number of classical transitions represented by one accelerated
odd step. -/
def acceleratedWeight (n : ℕ) : ℕ := 1 + nuTwo (3 * n + 1)

theorem acceleratedStep_pos {n : ℕ} (hn : 0 < n) : 0 < acceleratedStep n := by
  exact oddPart_pos (three_mul_add_one_pos hn)

theorem acceleratedStep_odd {n : ℕ} (hn : 0 < n) : acceleratedStep n % 2 = 1 := by
  exact oddPart_odd (three_mul_add_one_pos hn)

/-- An accelerated odd step is one odd classical transition followed by the
exact `ν₂(3n+1)` classical halvings. -/
theorem accelerated_correspondence {n : ℕ} (_hn : 0 < n) (hodd : n % 2 = 1) :
    iterate (acceleratedWeight n) n = acceleratedStep n := by
  rw [acceleratedWeight, Nat.one_add, iterate_succ, classicalStep_of_odd hodd]
  exact iterate_nuTwo (3 * n + 1)

/-- One compressed checkpoint, totalized on naturals. Its public mathematical
contract below is stated only for positive nonterminal inputs. -/
def compressedStep (n : ℕ) : ℕ :=
  if n % 2 = 0 then oddPart n else acceleratedStep n

def compressedWeight (n : ℕ) : ℕ :=
  if n % 2 = 0 then nuTwo n else acceleratedWeight n

theorem compressedStep_pos {n : ℕ} (hn : 0 < n) : 0 < compressedStep n := by
  rcases mod_two_eq_zero_or_one n with heven | hodd
  · simp [compressedStep, heven, oddPart_pos hn]
  · simp [compressedStep, show n % 2 ≠ 0 by omega, acceleratedStep_pos hn]

private theorem compressed_checkpoint_eq_iterate_of_pos {n : ℕ} (hn : 0 < n) :
    compressedStep n = iterate (compressedWeight n) n := by
  rcases mod_two_eq_zero_or_one n with heven | hodd
  · simp [compressedStep, compressedWeight, heven, (iterate_nuTwo n).symm]
  · simp [compressedStep, compressedWeight, show n % 2 ≠ 0 by omega,
      (accelerated_correspondence hn hodd).symm]

/-- For every positive nonterminal checkpoint, one compressed iteration is
the classical iterate selected by its exact classical weight. -/
theorem compressed_checkpoint_correspondence {n : ℕ} (hn : 1 < n) :
    compressedStep n = iterate (compressedWeight n) n :=
  compressed_checkpoint_eq_iterate_of_pos (by omega)

def compressedIterate : ℕ → ℕ → ℕ
  | 0, n => n
  | r + 1, n => compressedIterate r (compressedStep n)

def accumulatedWeight : ℕ → ℕ → ℕ
  | 0, _ => 0
  | r + 1, n => compressedWeight n + accumulatedWeight r (compressedStep n)

/-- Summed compressed weights are exactly the corresponding number of
classical transitions. -/
theorem compressed_accumulated_accounting {n : ℕ} (hn : 0 < n) (r : ℕ) :
    compressedIterate r n = iterate (accumulatedWeight r n) n := by
  induction r generalizing n with
  | zero => rfl
  | succ r ih =>
      simp only [compressedIterate, accumulatedWeight]
      rw [ih (compressedStep_pos hn)]
      rw [compressed_checkpoint_eq_iterate_of_pos hn]
      rw [← iterate_add]

theorem iterate_pow_two_mul_prefix {j k q : ℕ} (hj : j ≤ k) :
    iterate j (2 ^ k * q) = 2 ^ (k - j) * q := by
  induction j generalizing k with
  | zero => simp
  | succ j ih =>
      cases k with
      | zero => omega
      | succ k =>
          rw [iterate_succ]
          have hstep : classicalStep (2 ^ (k + 1) * q) = 2 ^ k * q := by
            simpa [pow_succ, Nat.mul_assoc, Nat.mul_comm, Nat.mul_left_comm] using
              classicalStep_two_mul (2 ^ k * q)
          rw [hstep]
          simpa using ih (k := k) (by omega)

theorem iterate_pow_two_mul_prefix_le {j k q : ℕ} (hj : j ≤ k) :
    iterate j (2 ^ k * q) ≤ 2 ^ k * q := by
  rw [iterate_pow_two_mul_prefix hj]
  exact Nat.mul_le_mul_right q (pow_le_pow_right' (by norm_num) (Nat.sub_le k j))

theorem iterate_nuTwo_prefix_le {n j : ℕ} (hj : j ≤ nuTwo n) : iterate j n ≤ n := by
  calc
    iterate j n = iterate j (2 ^ padicValNat 2 n * n.divMaxPow 2) := by
      rw [Nat.pow_padicValNat_mul_divMaxPow]
    _ ≤ 2 ^ padicValNat 2 n * n.divMaxPow 2 := by
      exact iterate_pow_two_mul_prefix_le (by simpa [nuTwo] using hj)
    _ = n := Nat.pow_padicValNat_mul_divMaxPow 2 n

/-- The only possible peak candidate skipped by an odd compressed macro is its
first classical value `3n+1`; an even macro only decreases. -/
def macroPeakCandidate (n : ℕ) : ℕ :=
  if n % 2 = 0 then n else max n (3 * n + 1)

theorem odd_macro_skipped_value_is_peak {n : ℕ} (hn : 0 < n) (hodd : n % 2 = 1) :
    iterate 1 n = macroPeakCandidate n := by
  rw [iterate_succ, iterate_zero, classicalStep_of_odd hodd]
  have hodd_ne : n % 2 ≠ 0 := by omega
  have hle : n ≤ 3 * n + 1 := by nlinarith
  simp only [macroPeakCandidate, if_neg hodd_ne, max_eq_right hle]

/-- Exact one-macro peak characterization: every represented classical value
is bounded by `macroPeakCandidate`, and that candidate is itself represented. -/
theorem compressed_macro_peak_exact {n : ℕ} (hn : 1 < n) :
    (∀ j ≤ compressedWeight n, iterate j n ≤ macroPeakCandidate n) ∧
      ∃ j ≤ compressedWeight n, iterate j n = macroPeakCandidate n := by
  rcases mod_two_eq_zero_or_one n with heven | hodd
  · constructor
    · intro j hj
      simpa [compressedWeight, macroPeakCandidate, heven] using
        iterate_nuTwo_prefix_le (n := n) (j := j) (by simpa [compressedWeight, heven] using hj)
    · refine ⟨0, ?_, ?_⟩
      · simp
      · simp [macroPeakCandidate, heven]
  · have hodd_ne : n % 2 ≠ 0 := by omega
    constructor
    · intro j hj
      cases j with
      | zero => simp [macroPeakCandidate, hodd_ne]
      | succ j =>
          have hj' : j ≤ nuTwo (3 * n + 1) := by
            have hweight : j + 1 ≤ 1 + nuTwo (3 * n + 1) := by
              simpa [compressedWeight, acceleratedWeight, hodd_ne] using hj
            omega
          rw [iterate_succ, classicalStep_of_odd hodd]
          exact (iterate_nuTwo_prefix_le (n := 3 * n + 1) (j := j) hj').trans (by
            simp [macroPeakCandidate, hodd_ne])
    · refine ⟨1, ?_, odd_macro_skipped_value_is_peak (by omega) hodd⟩
      simp [compressedWeight, acceleratedWeight, hodd_ne]

end Collatz
