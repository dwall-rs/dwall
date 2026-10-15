import { createMemo, createSignal, mergeProps, splitProps } from "solid-js";
import type { SwitchProps } from "./Switch.types";
import { callEventHandler } from "~/utils/call-event-handler";
import { clsx } from "~/utils";
import { classes } from "./Switch.styles";

export const Switch = (props: SwitchProps) => {
  const merged = mergeProps({ size: "md" } as const, props);

  const [local, others] = splitProps(merged, [
    "id",
    "checked",
    "defaultChecked",
    "onCheckedChange",
    "onClick",
    "onKeyDown",
    "size",
    "disabled",
    "class",
    "classList",
  ]);

  const [internalChecked, setInternalChecked] = createSignal(
    local.defaultChecked ?? false,
  );

  const toggle = () => {
    if (local.disabled) return;

    if (local.checked !== undefined) {
      setInternalChecked(local.checked);
      local.onCheckedChange?.(!local.checked);
      return;
    }

    const newChecked = !internalChecked();
    setInternalChecked(newChecked);
    local.onCheckedChange?.(newChecked);
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
      tabIndex={0}
      aria-checked={checked()}
      aria-disabled={local.disabled}
      class={clsx(classes.switch, local.class)}
      // `onClick`/`onKeyDown` are split out of `others` on purpose: spreading the
      // rest after this handler used to overwrite it, which made the switch inert
      // whenever a consumer passed `onClick` (e.g. to stop propagation inside a
      // clickable card).
      onClick={(event) => {
        callEventHandler(local.onClick, event);
        toggle();
      }}
      onKeyDown={(event) => {
        callEventHandler(local.onKeyDown, event);

        if (event.key === " " || event.key === "Enter") {
          event.preventDefault();
          toggle();
        }
      }}
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
