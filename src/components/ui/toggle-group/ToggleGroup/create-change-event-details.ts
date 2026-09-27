import type { ToggleGroupChangeEventDetails } from "./ToggleGroup.types";

/**
 * Builds the event details for onValueChange (aligned with base-ui's
 * `ChangeEventDetails`).
 *
 * `isCanceled` / `isPropagationAllowed` are exposed as getters: inside the
 * callback the latest value set by `cancel()` / `allowPropagation()` is read
 * immediately, not a snapshot from creation time.
 */
export function createChangeEventDetails(
  event: Event,
  trigger?: Element,
): ToggleGroupChangeEventDetails {
  let canceled = false;
  let propagationAllowed = false;

  return {
    reason: "none",
    event,
    trigger,
    cancel: () => {
      canceled = true;
    },
    allowPropagation: () => {
      propagationAllowed = true;
    },
    get isCanceled() {
      return canceled;
    },
    get isPropagationAllowed() {
      return propagationAllowed;
    },
  };
}
