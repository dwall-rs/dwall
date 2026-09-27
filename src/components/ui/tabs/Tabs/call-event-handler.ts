/**
 * Invoke a user-supplied event handler (supports Solid's function or
 * function-array forms). The type extracted from splitProps restKeys is an
 * EventHandlerUnion and cannot be called directly.
 */
export function callEventHandler<E extends Event>(
  handler: unknown,
  event: E,
): void {
  const list = Array.isArray(handler) ? handler : [handler];
  for (const h of list) {
    if (typeof h === "function") (h as (e: E) => void)(event);
  }
}
