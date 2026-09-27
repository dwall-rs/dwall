import { createMemo } from "solid-js";
import { useSelectContext } from "../Select/Select.context";
import type { SelectValueProps } from "./SelectValue.types";
import { clsx } from "~/utils";

export function SelectValue(props: SelectValueProps) {
  const ctx = useSelectContext("SelectValue");

  const label = createMemo(() => {
    const v = ctx.value();
    if (v === undefined || v === null) return undefined;
    return ctx.items.find((it) => it.value === v)?.label ?? v;
  });

  return (
    <span class={clsx(!label() && "text-neutral-500", props.class)}>
      {label() ?? props.placeholder ?? ""}
    </span>
  );
}
