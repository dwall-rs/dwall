import { splitProps } from "solid-js";
import { clsx } from "~/utils";
import type { FieldLabelProps } from "./FieldLabel.types";
import { Label } from "~/components/label";

export const FieldLabel = (props: FieldLabelProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <Label
      data-slot="field-label"
      class={clsx(
        "group/field-label peer/field-label flex w-fit gap-2 leading-snug group-data-[disabled=true]/field:opacity-50 has-data-[checked=true]:border-neutral-900/30 has-data-[checked=true]:bg-neutral-900/5 has-[>[data-slot=field]]:rounded-lg has-[>[data-slot=field]]:border *:data-[slot=field]:p-2.5 dark:has-data-[checked=true]:border-neutral-200/20 dark:has-data-[checked=true]:bg-neutral-200/10",
        "has-[>[data-slot=field]]:w-full has-[>[data-slot=field]]:flex-col",
        local.class,
      )}
      {...others}
    />
  );
};
