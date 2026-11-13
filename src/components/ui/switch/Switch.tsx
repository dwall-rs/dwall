import { createMemo, createSignal, mergeProps, splitProps } from "solid-js";
import type { SwitchProps } from "./Switch.types";
import { callEventHandler, clsx } from "~/utils";
import { classes } from "./Switch.styles";

export const Switch = (props: SwitchProps) => {
  const merged = mergeProps({ size: "md" } as const, props);

  const [local, events, others] = splitProps(
    merged,
    [
      "id",
      "checked",
      "defaultChecked",
      "onCheckedChange",
      "size",
      "disabled",
      "class",
      "classList",
    ],
    ["onClick", "onKeyDown"],
  );

  const [internalChecked, setInternalChecked] = createSignal(
    local.defaultChecked ?? false,
  );

  const toggle = () => {
    if (local.checked !== undefined) {
      setInternalChecked(local.checked);
      local.onCheckedChange?.(!local.checked);
      return;
    }

    const newChecked = !internalChecked();
    setInternalChecked(newChecked);
    local.onCheckedChange?.(newChecked);
  };

  const handleClick = (event: MouseEvent) => {
    callEventHandler(events.onClick, event);
    if (local.disabled) return;
    toggle();
  };

  const handleKeyDown = (event: KeyboardEvent) => {
    callEventHandler(events.onKeyDown, event);
    if (event.defaultPrevented || local.disabled) return;
    if (event.key !== " " && event.key !== "Enter") return;

    event.preventDefault();
    if (event.repeat) return;
    toggle();
  };

  const checked = createMemo(() =>
    local.checked !== undefined ? local.checked : internalChecked(),
  );

  return (
    <span
      data-slot="switch"
      data-size={local.size}
      data-checked={checked()}
      data-disabled={local.disabled}
      role="switch"
      tabIndex={local.disabled ? -1 : 0}
      aria-checked={checked()}
      aria-disabled={local.disabled}
      class={clsx(classes.switch, local.class)}
      classList={local.classList}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      {...others}
    >
      <span
        data-slot="switch-thumb"
        class={classes.thumb}
        data-checked={checked()}
      />
      <input
        id={local.id}
        tabIndex={-1}
        aria-hidden="true"
        type="checkbox"
        checked={checked()}
        class="sr-only"
        disabled={local.disabled}
      />
    </span>
  );
};
