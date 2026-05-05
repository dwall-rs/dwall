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
        "h-8 w-full min-w-0 rounded-lg border border-neutral-200 dark:border-white/15 bg-transparent px-2.5 py-1 text-base transition-colors outline-none file:inline-flex file:h-6 file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-neutral-950 dark:file:text-neutral-50 placeholder:text-neutral-500 dark:placeholder:text-neutral-400 focus-visible:border-neutral-400 dark:focus-visible:border-neutral-500 focus-visible:ring-3 focus-visible:ring-neutral-400/50 dark:focus-visible:ring-neutral-500/50 disabled:pointer-events-none disabled:cursor-not-allowed disabled:select-none disabled:bg-neutral-200/50 disabled:opacity-50 aria-invalid:border-red-600 aria-invalid:ring-3 aria-invalid:ring-red-600/20 md:text-sm dark:bg-white/4.5 dark:disabled:bg-white/12 dark:aria-invalid:border-red-400/50 dark:aria-invalid:ring-red-400/40",
        local.class,
      )}
      onInput={handleInput}
      onChange={handleChange}
      {...others}
    />
  );
};
