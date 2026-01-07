/**
 * Calculate the experience required to reach a specific level.
 * This mirrors the backend's experience_for_level function.
 */
export function experienceForLevel(level: number): number {
  if (level <= 1) {
    return 0;
  }

  const baseExp = 100.0;
  const exponent = 1.5;

  return Math.round(baseExp * Math.pow(level, exponent));
}

/**
 * Calculate what level a character should be based on their total experience.
 * Returns [currentLevel, experienceIntoCurrentLevel, experienceNeededForNextLevel]
 *
 * This mirrors the backend's calculate_level_from_experience function.
 */
export function calculateLevelFromExperience(
  totalExp: number
): [number, number, number] {
  let level = 1;
  let accumulatedExp = 0;

  // Keep leveling up until we run out of experience
  while (true) {
    const expForNext = experienceForLevel(level + 1);
    if (accumulatedExp + expForNext > totalExp) {
      // Not enough experience for next level
      const expIntoLevel = totalExp - accumulatedExp;
      return [level, expIntoLevel, expForNext];
    }

    accumulatedExp += expForNext;
    level += 1;

    // Safety cap at level 100
    if (level >= 100) {
      return [100, 0, Number.MAX_SAFE_INTEGER];
    }
  }
}
