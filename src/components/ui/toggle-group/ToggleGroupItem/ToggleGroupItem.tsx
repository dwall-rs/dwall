import { mergeProps, onCleanup, onMount, splitProps, type JSX } from "solid-js";
import { clsx } from "~/utils";
import { toggleVariants } from "~/components/ui/toggle/Toggle/Toggle.styles";
import { callEventHandler } from "../call-event-handler";
import { useToggleGroupContext } from "../ToggleGroup/ToggleGroup.context";
import type { ToggleGroupValue } from "../ToggleGroup/ToggleGroup.types";
import { toggleGroupItemVariants } from "./ToggleGroupItem.styles";
import type { ToggleGroupItemProps } from "./ToggleGroupItem.types";

/**
 * ToggleGroupItem: a two-state button inside a ToggleGroup.
 *
 * - The selected value is held centrally by ToggleGroup; clicking triggers a
 *   change via `ctx.toggleItem`
 * - disabled inherits the whole group's state and can also be set per item
 * - roving tabindex: the group has a single tab stop, arrow keys move focus
 *   within the group
 */
export function ToggleGroupItem<
  TValue extends ToggleGroupValue = ToggleGroupValue,
>(props: ToggleGroupItemProps<TValue>): JSX.Element {
  // The context is read with the wide value type: an item cannot derive the
  // group's value type from its parent, while its own TValue is always
  // assignable to the wide type parameter
  const ctx = useToggleGroupContext("ToggleGroupItem");
  const merged = mergeProps({ type: "button" as const }, props);

  const [local, rest] = splitProps(merged, [
    "value",
    "variant",
    "size",
    "class",
    "classList",
    "disabled",
    "type",
    "onClick",
    "onFocus",
    "children",
  ]);

  let elementRef: HTMLButtonElement | undefined;

  // The item can override variant/size individually when the group does not specify them
  const variant = () => ctx.variant() ?? local.variant ?? "default";
  const size = () => ctx.size() ?? local.size ?? "default";
  const pressed = () => ctx.isPressed(local.value);
  const disabled = () => ctx.disabled() || !!local.disabled;

  // roving tabindex: the highlighted item holds 0; when the highlight is
  // missing or points at an unusable item (disabled/unmounted), the first
  // enabled item holds 0 as the keyboard starting point, so the group is
  // never unreachable via the keyboard
  const tabIndex = () => {
    if (disabled()) return -1;
    const highlighted = ctx.highlightedValue();
    if (highlighted === local.value && ctx.isUsable(local.value)) return 0;
    if (
      (highlighted === undefined || !ctx.isUsable(highlighted)) &&
      ctx.isFirstEnabled(local.value)
    ) {
      return 0;
    }
    return -1;
  };

  onMount(() => {
    onCleanup(
      ctx.registerItem({
        value: local.value,
        disabled,
        element: elementRef!,
      }),
    );
  });

  return (
    <button
      ref={(el) => {
        elementRef = el;
      }}
      type={local.type}
      disabled={disabled()}
      tabindex={tabIndex()}
      data-slot="toggle-group-item"
      data-variant={variant()}
      data-size={size()}
      data-spacing={ctx.spacing()}
      data-pressed={pressed() ? "" : null}
      data-disabled={disabled() ? "" : null}
      data-state={pressed() ? "on" : "off"}
      aria-pressed={pressed()}
      class={clsx(
        toggleVariants({ variant: variant(), size: size() }),
        toggleGroupItemVariants(),
        local.class,
      )}
      classList={local.classList}
      onClick={(e) => {
        // The user callback takes priority; preventDefault() can block this selection
        callEventHandler(local.onClick, e);
        if (!e.defaultPrevented && !disabled()) {
          ctx.toggleItem(local.value, e, e.currentTarget);
        }
      }}
      onFocus={(e) => {
        // Focus updates the highlight (the roving focus starting point)
        if (!disabled()) ctx.setHighlightedValue(local.value);
        callEventHandler(local.onFocus, e);
      }}
      {...rest}
    >
      {local.children}
    </button>
  );
}
