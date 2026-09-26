import type { ToggleGroupChangeEventDetails } from "./ToggleGroup.types";

/**
 * 构造 onValueChange 的事件详情(对齐 base-ui 的 `ChangeEventDetails`)。
 *
 * `isCanceled` / `isPropagationAllowed` 用 getter 暴露:回调内部可以即时读到
 * `cancel()` / `allowPropagation()` 之后的最新值,而不是创建时的快照。
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
