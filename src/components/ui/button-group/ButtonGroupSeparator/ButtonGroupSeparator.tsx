import { mergeProps } from "solid-js";
import { Separator } from "~/components/ui/separator";
import { clsx } from "~/utils";
import classes from "./ButtonGroupSeparator.styles";
import type { ButtonGroupSeparatorProps } from "./ButtonGroupSeparator.types";

export const ButtonGroupSeparator = (props: ButtonGroupSeparatorProps) => {
  const merged = mergeProps({ orientation: "vertical" } as const, props);

  return (
    <Separator
      {...merged}
      data-slot="button-group-separator"
      orientation={merged.orientation}
      class={clsx(classes, merged.class)}
    />
  );
};
