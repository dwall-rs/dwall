import { splitProps, type JSX } from "solid-js";
import { clsx } from "~/utils";
import { callEventHandler } from "../call-event-handler";
import {
  createToggleGroupState,
  ToggleGroupContext,
} from "./ToggleGroup.context";
import { toggleGroupVariants } from "./ToggleGroup.styles";
import type { ToggleGroupProps, ToggleGroupValue } from "./ToggleGroup.types";
import { useToggleGroupKeyboard } from "./useToggleGroupKeyboard";

/**
 * ToggleGroup root component: renders `<div role="group">`, emits styling and
 * semantic hooks (`data-orientation` / `data-vertical` / `data-horizontal` /
 * `data-multiple` / `data-disabled` / `data-spacing` / `data-variant` /
 * `data-size`), and mounts roving focus keyboard navigation.
 *
 * In single mode (default) value is a scalar; only multiple mode
 * (`multiple`) uses an array:
 *
 * @example
 * ```tsx
 * // single
 * <ToggleGroup defaultValue="bold" onValueChange={(v) => ...}>
 *   <ToggleGroupItem value="bold">Bold</ToggleGroupItem>
 * </ToggleGroup>
 *
 * // multiple
 * <ToggleGroup multiple value={["bold"]} onValueChange={(v) => ...}>
 *   <ToggleGroupItem value="bold">Bold</ToggleGroupItem>
 * </ToggleGroup>
 * ```
 */
export function ToggleGroup<TValue extends ToggleGroupValue = ToggleGroupValue>(
  props: ToggleGroupProps<TValue>,
): JSX.Element {
  const [local, rest] = splitProps(props, [
    "value",
    "defaultValue",
    "onValueChange",
    "multiple",
    "disabled",
    "orientation",
    "loopFocus",
    "spacing",
    "variant",
    "size",
    "class",
    "classList",
    "style",
    "dir",
    "onKeyDown",
    "children",
  ]);

  // Defaults and the single/multiple differences are handled uniformly by the
  // state layer; rendering uses the resolved value from ctx
  const ctx = createToggleGroupState<TValue>(local);
  const { handleKeyDown } = useToggleGroupKeyboard(ctx);
  const isVertical = () => ctx.orientation() === "vertical";

  return (
    <div
      role="group"
      data-slot="toggle-group"
      data-variant={ctx.variant()}
      data-size={ctx.size()}
      data-spacing={ctx.spacing()}
      data-orientation={ctx.orientation()}
      // Joined-state styles depend on data-vertical / data-horizontal (consistent with the tabs root)
      data-vertical={isVertical() ? "" : null}
      data-horizontal={isVertical() ? null : ""}
      data-multiple={ctx.multiple() ? "" : null}
      data-disabled={ctx.disabled() ? "" : null}
      aria-disabled={ctx.disabled() ? "true" : undefined}
      dir={local.dir}
      // --gap is the styling entry point for spacing; overridable via user style
      style={{ "--gap": ctx.spacing(), ...local.style } as JSX.CSSProperties}
      class={clsx(toggleGroupVariants(), local.class)}
      classList={local.classList}
      onKeyDown={(e) => {
        callEventHandler(local.onKeyDown, e);
        if (!e.defaultPrevented) handleKeyDown(e);
      }}
      {...rest}
    >
      <ToggleGroupContext.Provider value={ctx}>
        {local.children}
      </ToggleGroupContext.Provider>
    </div>
  );
}
