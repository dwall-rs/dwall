/**
 * Compose several Solid refs into one callback ref. Solid only invokes
 * callback refs, so non-function entries are ignored.
 */
export function mergeRefs<T>(
  ...refs: Array<T | ((el: T) => void) | undefined>
): (el: T) => void {
  return (el: T) => {
    for (const ref of refs) {
      if (typeof ref === "function") (ref as (el: T) => void)(el);
    }
  };
}
