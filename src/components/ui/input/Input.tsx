import {
  createMemo,
  createUniqueId,
  type JSX,
  mergeProps,
  splitProps,
} from "solid-js";
import type { InputProps } from "./Input.types";
import { clsx } from "~/utils";

export const Input = (props: InputProps) => {
  const merged = mergeProps(
    { id: createUniqueId(), type: "text" } as const,
    props,
  );

  const [local, others] = splitProps(merged, [
    "value",
    "defaultValue",
    "type",
    "onInput",
    "onChange",
    "class",
  ]);

  const handleInput: JSX.IntrinsicElements["input"]["onInput"] = (e) => {
    let value: string | number = e.target.value;
    if (local.type === "number") {
      value = Number(e.target.value);
    }
    // @ts-expect-error
    local.onInput?.(value);
  };

  const handleChange: JSX.IntrinsicElements["input"]["onChange"] = (e) => {
    let value: string | number = e.target.value;
    if (local.type === "number") {
      value = Number(e.target.value);
    }
    // @ts-expect-error
    local.onChange?.(value);
  };

  const value = createMemo(() =>
    local.value !== undefined ? local.value : local.defaultValue,
  );

  return (
    <input
      type={local.type}
      value={value()}
      data-slot="input"
      class={clsx(
        "h-8 w-full min-w-0 rounded-lg border border-input bg-transparent px-2.5 py-1 text-base transition-colors outline-none file:inline-flex file:h-6 file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-foreground placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:cursor-not-allowed disabled:bg-input/50 disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-3 aria-invalid:ring-destructive/20 md:text-sm dark:bg-input/30 dark:disabled:bg-input/80 dark:aria-invalid:border-destructive/50 dark:aria-invalid:ring-destructive/40",
        local.class,
      )}
      onInput={handleInput}
      onChange={handleChange}
      {...others}
    />
  );
};
