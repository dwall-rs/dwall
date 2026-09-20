import type { ParentProps } from "solid-js";

export interface SelectLabelProps extends ParentProps {
  class?: string;
}

export function SelectLabel(props: SelectLabelProps) {
  return (
    <div
      class={`px-2 py-1.5 text-xs font-medium text-neutral-500 ${props.class ?? ""}`}
    >
      {props.children}
    </div>
  );
}
