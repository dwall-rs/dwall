import { splitProps } from "solid-js";
import { clsx } from "~/utils";
import type { FieldLabelProps } from "./FieldLabel.types";
import { Label } from "~/components/ui/label";

export const FieldLabel = (props: FieldLabelProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <Label
      data-slot="field-label"
      class={clsx(
        "group/field-label peer/field-label flex w-fit gap-2 leading-snug group-data-[disabled=true]/field:opacity-50 has-data-[checked=true]:border-primary/30 has-data-[checked=true]:bg-primary/5 has-[>[data-slot=field]]:rounded-lg has-[>[data-slot=field]]:border *:data-[slot=field]:p-2.5 dark:has-data-[checked=true]:border-primary/20 dark:has-data-[checked=true]:bg-primary/10",
        "has-[>[data-slot=field]]:w-full has-[>[data-slot=field]]:flex-col",
        local.class,
      )}
      {...others}
    />
  );
};
