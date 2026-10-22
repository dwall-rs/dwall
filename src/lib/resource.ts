// Responsibility: read a Solid resource without letting its error state escape.

/** The part of a resource this helper needs; kept structural so a test needs no runtime. */
interface Readable<T> {
  (): T | undefined;
  error: unknown;
}

/**
 * Reads a resource, returning `undefined` while it is in its error state.
 *
 * Solid's resource accessor *throws* in the error state
 * (`if (err !== undefined && !pr) throw err`), and a render-time throw takes down the
 * whole reactive tree. Here a denied location permission or a corrupt theme file is an
 * expected outcome, not a crash, so callers get `undefined` and show their own fallback.
 */
export const resourceValue = <T>(resource: Readable<T>): T | undefined =>
  resource.error ? undefined : resource();
