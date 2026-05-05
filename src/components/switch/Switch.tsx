import { createMemo, createSignal, mergeProps, splitProps } from "solid-js";
import type { SwitchProps } from "./Switch.types";
import { clsx } from "~/utils";
import { classes } from "./Switch.styles";

export const Switch = (props: SwitchProps) => {
  const merged = mergeProps({ size: "md" } as const, props);

  const [local, others] = splitProps(merged, [
    "id",
    "checked",
    "defaultChecked",
    "onCheckedChange",
    "size",
    "disabled",
    "class",
    "classList",
  ]);

  const [internalChecked, setInternalChecked] = createSignal(
    local.defaultChecked,
  );

  const handleClick = () => {
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
      class={clsx(classes.switch, local.class)}
      onClick={handleClick}
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
