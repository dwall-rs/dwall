import { splitProps } from "solid-js";
import type { SelectGroupProps } from "./SelectGroup.types";
import { clsx } from "~/utils";

export function SelectGroup(props: SelectGroupProps) {
  const [local, others] = splitProps(props, ["class", "classList", "children"]);

  return (
    <li
      data-slot="select-group"
      role="group"
      class={clsx(local.class)}
      classList={local.classList}
      {...others}
    >
      {local.children}
    </li>
  );
}
