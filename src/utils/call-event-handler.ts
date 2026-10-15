/**
 * Invoke a user-supplied event handler.
 *
 * `splitProps` surfaces Solid's `EventHandlerUnion`, which cannot be called
 * directly: it is either a plain handler or a bound `[data, handler]` tuple.
 *
 * Components that need to run their own logic *and* respect a consumer-supplied
 * handler (e.g. `onClick` used to stop propagation inside a clickable card) must
 * extract that prop and call it through this helper — spreading the remaining
 * props after an internal handler silently overwrites it.
 */
export function callEventHandler<E extends Event>(
  handler: unknown,
  event: E,
): void {
  if (typeof handler === "function") {
    (handler as (event: E) => void)(event);
    return;
  }

  if (Array.isArray(handler)) {
    // Bound handler form: [data, handler].
    const [data, bound] = handler as [unknown, unknown];
    if (typeof bound === "function") {
      (bound as (data: unknown, event: E) => void)(data, event);
    }
  }
}
