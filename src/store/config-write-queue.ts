// Responsibility: one queue for every operation that writes the configuration file.
//
// Three commands write the same file from this window: a settings save, applying a
// wallpaper mode, and stopping the engine (which clears the mode). They used to run
// independently, so a save that was already in flight could land after a stop and put
// the cleared mode back, and an apply could overwrite a newer saved snapshot with one
// it captured before the save finished. Serializing them makes the last user action the
// last write, and lets each operation read the snapshot the previous one committed.

let tail: Promise<unknown> | undefined;

/**
 * Runs `task` after every configuration write queued before it.
 *
 * With nothing queued the task starts immediately, so callers keep their previous
 * "saving" state in the same tick. A rejected task does not stop the queue: later
 * writes still run, in order.
 */
export const queueConfigWrite = <T>(task: () => Promise<T>): Promise<T> => {
  const previous = tail;
  const run = previous ? previous.then(task, task) : task();

  let settled: Promise<T>;
  settled = run.then(
    (value) => {
      if (tail === settled) tail = undefined;
      return value;
    },
    (error: unknown) => {
      if (tail === settled) tail = undefined;
      throw error;
    },
  );

  tail = settled;
  return settled;
};
