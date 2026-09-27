import {
  createMemo,
  createSignal,
  mergeProps,
  splitProps,
  type JSX,
} from "solid-js";
import { clsx } from "~/utils";
import { toggleVariants } from "./Toggle.styles";
import type { ToggleProps } from "./Toggle.types";

export function Toggle(props: ToggleProps): JSX.Element {
  const merged = mergeProps({ type: "button" as const }, props);

  const [local, rest] = splitProps(merged, [
    "pressed",
    "defaultPressed",
    "onPressedChange",
    "variant",
    "size",
    "class",
    "value",
    "type",
    "onClick",
  ]);

  const [internalPressed, setInternalPressed] = createSignal(
    local.defaultPressed ?? false,
  );
  const pressed = createMemo(() =>
    local.pressed !== undefined ? local.pressed : internalPressed(),
  );

  const setPressed = (next: boolean) => {
    if (local.pressed === undefined) setInternalPressed(next);
    local.onPressedChange?.(next);
  };

  return (
    <button
      type={local.type as "button"}
      data-slot="toggle"
      data-state={pressed() ? "on" : "off"}
      aria-pressed={pressed()}
      class={clsx(
        toggleVariants({
          variant: local.variant,
          size: local.size,
          class: local.class,
        }),
      )}
      onClick={(e) => {
        (local.onClick as any)?.(e);
        if (!e.defaultPrevented) setPressed(!pressed());
      }}
      {...rest}
    />
  );
}
