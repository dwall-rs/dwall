import { splitProps } from "solid-js";
import { clsx } from "~/utils";
import type { FieldTitleProps } from "./FieldTitle.types";

export const FieldTitle = (props: FieldTitleProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="field-label"
      class={clsx(
        "flex w-fit items-center gap-2 text-sm font-medium group-data-[disabled=true]/field:opacity-50",
        local.class,
      )}
      {...others}
    />
  );
};
