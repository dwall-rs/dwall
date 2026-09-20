import { createMemo } from "solid-js";
import { useSelectContext } from "../Select/Select.context";
import type { SelectValueProps } from "./SelectValue.types";

function cn(...classes: Array<string | false | undefined | null>): string {
  return classes.filter(Boolean).join(" ");
}

export function SelectValue(props: SelectValueProps) {
  const ctx = useSelectContext("SelectValue");

  const label = createMemo(() => {
    const v = ctx.value();
    if (v === undefined) return undefined;
    return ctx.items.find((it) => it.value === v)?.label ?? v;
  });

  return (
    <span class={cn(!label() && "text-neutral-500", props.class)}>
      {label() ?? props.placeholder ?? ""}
    </span>
  );
}
