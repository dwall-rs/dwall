/**
 * Calls a user-supplied event handler (supporting Solid's function / array of
 * functions / bound object forms). The event type extracted by splitProps is
 * an EventHandlerUnion and cannot be called directly.
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
