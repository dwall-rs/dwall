import { splitProps } from "solid-js";
import { clsx } from "~/utils";
import type { FieldGroupProps } from "./FieldGroup.types";

export const FieldGroup = (props: FieldGroupProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="field-group"
      class={clsx(
        "group/field-group @container/field-group flex w-full flex-col gap-5 data-[slot=checkbox-group]:gap-3 *:data-[slot=field-group]:gap-4",
        local.class,
      )}
      {...others}
    />
  );
};
