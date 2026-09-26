import { Label, type LabelProps } from "~/components/ui/label";
import { clsx } from "~/utils";

export function SelectLabel(props: LabelProps) {
  return (
    <Label
      {...props}
      class={clsx(
        "px-2 py-1.5 text-xs font-medium text-neutral-500",
        props.class,
      )}
    />
  );
}
