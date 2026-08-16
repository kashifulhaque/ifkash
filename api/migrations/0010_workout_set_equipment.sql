-- Record which implement each set was performed with.
--
-- The same exercise name was being logged against different equipment — machine
-- pec fly at 52.5 kg one day, dumbbell flyes at ~17 kg the next when the machine
-- was occupied — so the load progression for that exercise name was meaningless.
-- Splitting on equipment makes each implement its own progression line.
--
-- Existing rows keep '' (unspecified) rather than being guessed at.
ALTER TABLE workout_sets ADD COLUMN equipment TEXT NOT NULL DEFAULT '';

CREATE INDEX IF NOT EXISTS idx_workout_sets_exercise_equipment
  ON workout_sets (exercise, equipment);
