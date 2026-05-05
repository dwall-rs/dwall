import { mergeProps, splitProps } from "solid-js";
import { clsx } from "~/utils";
import type { FieldProps } from "./Field.types";
import { fieldVariants } from "./Field.styles";

export const Field = (props: FieldProps) => {
  const merged = mergeProps({ orientation: "vertical" } as const, props);

  const [local, others] = splitProps(merged, [
    "orientation",
    "class",
    "classList",
  ]);

  return (
    <div
      role="group"
      data-slot="field"
      data-orientation={local.orientation}
      class={clsx(
        fieldVariants({ orientation: local.orientation }),
        local.class,
      )}
      {...others}
    />
  );
};
