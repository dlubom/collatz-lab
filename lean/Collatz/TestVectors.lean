import Collatz.SpecialForms

namespace Collatz

theorem vector_one_trajectory : trajectoryValues 1 0 = [1] := by
  decide

theorem vector_one_observation :
    observe 1 0 = { status := .reachedOne, completedSteps := 0, last := 1 } := by
  decide

theorem vector_one_first_reaches_one : FirstReachesOneAt 1 0 := by
  unfold FirstReachesOneAt
  decide

theorem vector_one_peak : trajectoryPeak 1 0 = 1 := by
  decide

theorem vector_two_trajectory : trajectoryValues 2 1 = [2, 1] := by
  decide

theorem vector_two_observation :
    observe 2 1 = { status := .reachedOne, completedSteps := 1, last := 1 } := by
  decide

theorem vector_two_first_reaches_one : FirstReachesOneAt 2 1 := by
  unfold FirstReachesOneAt
  decide

theorem vector_two_peak : trajectoryPeak 2 1 = 2 := by
  decide

theorem vector_one_compressed_terminal :
    compressedIterate 0 1 = 1 ∧ accumulatedWeight 0 1 = 0 := by
  decide

theorem vector_one_compressed_accounting :
    compressedIterate 0 1 = iterate (accumulatedWeight 0 1) 1 := by
  apply compressed_accumulated_accounting (by decide)
  intro i hi
  omega

theorem vector_one_compressed_prefix_stops : ¬CompressedPrefixActive 1 1 := by
  intro hactive
  exact hactive 0 (by omega) rfl

theorem vector_two_compressed_terminal :
    compressedIterate 1 2 = 1 ∧ accumulatedWeight 1 2 = 1 := by
  simp [compressedIterate, compressedStep, accumulatedWeight, compressedWeight,
    oddPart, nuTwo, Nat.divMaxPow, padicValNat]

theorem vector_two_compressed_accounting :
    compressedIterate 1 2 = iterate (accumulatedWeight 1 2) 2 := by
  apply compressed_accumulated_accounting (by decide)
  intro i hi
  have hi_zero : i = 0 := by omega
  subst i
  simp [compressedIterate]

theorem vector_two_compressed_prefix_stops : ¬CompressedPrefixActive 2 2 := by
  intro hactive
  exact hactive 1 (by omega) vector_two_compressed_terminal.1

theorem vector_three_trajectory :
    trajectoryValues 3 7 = [3, 10, 5, 16, 8, 4, 2, 1] := by
  decide

theorem vector_three_observation :
    observe 3 7 = { status := .reachedOne, completedSteps := 7, last := 1 } := by
  decide

theorem vector_three_first_reaches_one : FirstReachesOneAt 3 7 := by
  unfold FirstReachesOneAt
  decide

theorem vector_three_peak : trajectoryPeak 3 7 = 16 := by
  decide

set_option maxRecDepth 10000 in
theorem vector_twenty_seven_observation :
    observe 27 111 = { status := .reachedOne, completedSteps := 111, last := 1 } := by
  decide

set_option maxRecDepth 10000 in
theorem vector_twenty_seven_first_reaches_one : FirstReachesOneAt 27 111 := by
  unfold FirstReachesOneAt
  decide

set_option maxRecDepth 10000 in
theorem vector_twenty_seven_peak : trajectoryPeak 27 111 = 9232 := by
  decide

theorem vector_mersenne_five : mersenne 5 = 31 := by
  decide

theorem vector_fermat_two : fermat 2 = 17 := by
  decide

theorem vector_repunit_ten_three : repunit 10 3 = 111 := by
  decide

theorem vector_affine_three_four : affinePowerTwo 3 4 = 47 := by
  decide

end Collatz
