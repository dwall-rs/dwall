import { mergeProps, splitProps } from "solid-js";
import { clsx } from "~/utils";
import type { FieldLegendProps } from "./FieldLegend.types";

export const FieldLegend = (props: FieldLegendProps) => {
  const merged = mergeProps({ variant: "legend" } as const, props);

  const [local, others] = splitProps(merged, ["variant", "class", "classList"]);

  return (
    <legend
      data-slot="field-legend"
      data-variant={local.variant}
      class={clsx(
        "mb-1.5 font-medium data-[variant=label]:text-sm data-[variant=legend]:text-base",
        local.class,
      )}
      {...others}
    />
  );
};
