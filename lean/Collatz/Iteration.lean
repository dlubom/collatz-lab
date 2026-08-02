import Collatz.Basic
import Mathlib.Data.Finset.Lattice.Fold
import Mathlib.Data.Finset.Range
import Mathlib.Data.List.Range

namespace Collatz

/-- `iterate k n` is `C^k(n)`. The starting value has index zero. -/
def iterate : ℕ → ℕ → ℕ
  | 0, n => n
  | k + 1, n => iterate k (classicalStep n)

@[simp]
theorem iterate_zero (n : ℕ) : iterate 0 n = n := rfl

@[simp]
theorem iterate_succ (k n : ℕ) : iterate (k + 1) n = iterate k (classicalStep n) := rfl

theorem iterate_add (a b n : ℕ) : iterate (a + b) n = iterate b (iterate a n) := by
  induction a generalizing n with
  | zero => simp
  | succ a ih =>
      rw [Nat.succ_add, iterate_succ, iterate_succ]
      exact ih (classicalStep n)

theorem iterate_pos {n : ℕ} (hn : 0 < n) (k : ℕ) : 0 < iterate k n := by
  induction k generalizing n with
  | zero => simpa
  | succ k ih =>
      rw [iterate_succ]
      exact ih (classicalStep_pos hn)

/-- The literal classical values from index zero through `steps`. -/
def trajectoryValues (start steps : ℕ) : List ℕ :=
  (List.range (steps + 1)).map fun k => iterate k start

@[simp]
theorem trajectoryValues_length (start steps : ℕ) :
    (trajectoryValues start steps).length = steps + 1 := by
  simp [trajectoryValues]

/-- A represented trajectory prefix through index `k` contains exactly `k`
completed classical transitions. -/
theorem classical_count_at_index (start k : ℕ) :
    (trajectoryValues start k).length - 1 = k := by
  simp

/-- Reachability records the index, hence also the number of completed
classical transitions. -/
def ReachableAt (start value index : ℕ) : Prop := iterate index start = value

theorem reachableAt_zero (n : ℕ) : ReachableAt n n 0 := rfl

theorem reachableAt_succ {start value : ℕ} {index : ℕ}
    (h : ReachableAt (classicalStep start) value index) :
    ReachableAt start value (index + 1) := by
  exact h

inductive ObservationStatus
  | reachedOne
  | stepLimitReached
  deriving DecidableEq, Repr

structure Observation where
  status : ObservationStatus
  completedSteps : ℕ
  last : ℕ
  deriving DecidableEq, Repr

/-- A bounded runner that checks for terminal `1` before checking whether the
classical-transition budget is exhausted. -/
def observe : ℕ → ℕ → Observation
  | n, 0 =>
      if n = 1 then
        { status := .reachedOne, completedSteps := 0, last := n }
      else
        { status := .stepLimitReached, completedSteps := 0, last := n }
  | n, limit + 1 =>
      if n = 1 then
        { status := .reachedOne, completedSteps := 0, last := n }
      else
        let tail := observe (classicalStep n) limit
        { tail with completedSteps := tail.completedSteps + 1 }

@[simp]
theorem observe_one (limit : ℕ) :
    observe 1 limit = { status := .reachedOne, completedSteps := 0, last := 1 } := by
  cases limit <;> simp [observe]

theorem observe_one_zero_terminal_before_limit :
    observe 1 0 = { status := .reachedOne, completedSteps := 0, last := 1 } := by
  simp

theorem observe_completedSteps_le (n limit : ℕ) : (observe n limit).completedSteps ≤ limit := by
  induction limit generalizing n with
  | zero =>
      by_cases hn : n = 1 <;> simp [observe, hn]
  | succ limit ih =>
      by_cases hn : n = 1
      · simp [observe, hn]
      · simp only [observe, hn, ↓reduceIte]
        exact Nat.succ_le_succ (ih (classicalStep n))

theorem observe_last_eq_iterate (n limit : ℕ) :
    (observe n limit).last = iterate (observe n limit).completedSteps n := by
  induction limit generalizing n with
  | zero =>
      by_cases hn : n = 1 <;> simp [observe, hn]
  | succ limit ih =>
      by_cases hn : n = 1
      · simp [observe, hn]
      · simp only [observe, hn, ↓reduceIte]
        rw [iterate_succ]
        exact ih (classicalStep n)

theorem observe_reachedOne_last {n limit : ℕ}
    (h : (observe n limit).status = .reachedOne) : (observe n limit).last = 1 := by
  induction limit generalizing n with
  | zero =>
      by_cases hn : n = 1 <;> simp [observe, hn] at h ⊢
  | succ limit ih =>
      by_cases hn : n = 1
      · simp [observe, hn]
      · simp only [observe, hn, ↓reduceIte] at h ⊢
        exact ih h

/-- The maximum over the represented classical prefix. -/
def trajectoryPeak (start steps : ℕ) : ℕ :=
  (Finset.range (steps + 1)).sup fun k => iterate k start

/-- Exact first arrival at `1`, used by the reviewed small vectors. -/
def earlierValuesAvoidOne (start index : ℕ) : Bool :=
  (List.range index).all fun j => decide (iterate j start ≠ 1)

theorem earlierValuesAvoidOne_eq_true_iff (start index : ℕ) :
    earlierValuesAvoidOne start index = true ↔ ∀ j < index, iterate j start ≠ 1 := by
  simp [earlierValuesAvoidOne]

def FirstReachesOneAt (start index : ℕ) : Prop :=
  iterate index start = 1 ∧ earlierValuesAvoidOne start index = true

theorem firstReachesOneAt_iff (start index : ℕ) :
    FirstReachesOneAt start index ↔
      iterate index start = 1 ∧ ∀ j < index, iterate j start ≠ 1 := by
  simp [FirstReachesOneAt, earlierValuesAvoidOne_eq_true_iff]

end Collatz
