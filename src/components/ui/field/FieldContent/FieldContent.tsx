import { splitProps } from "solid-js";
import { clsx } from "~/utils";
import type { FieldContentProps } from "./FieldContent.types";

export const FieldContent = (props: FieldContentProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="field-content"
      class={clsx(
        "group/field-content flex flex-1 flex-col gap-0.5 leading-snug",
        local.class,
      )}
      {...others}
    />
  );
};
