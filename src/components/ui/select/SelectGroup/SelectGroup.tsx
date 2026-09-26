import type { ParentProps } from "solid-js";

export function SelectGroup(props: ParentProps) {
  return <li role="group">{props.children}</li>;
}
