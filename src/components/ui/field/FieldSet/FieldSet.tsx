import { splitProps } from "solid-js";
import type { FieldSetProps } from "./FieldSet.types";
import { clsx } from "~/utils";

export const FieldSet = (props: FieldSetProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <fieldset
      data-slot="field-set"
      class={clsx(
        "flex flex-col gap-4 has-[>[data-slot=checkbox-group]]:gap-3 has-[>[data-slot=radio-group]]:gap-3",
        local.class,
      )}
      {...others}
    />
  );
};
