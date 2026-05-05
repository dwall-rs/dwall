import { splitProps } from "solid-js";
import { clsx } from "~/utils";
import type { FieldDescriptionProps } from "./FieldDescription.types";

export const FieldDescription = (props: FieldDescriptionProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <p
      data-slot="field-description"
      class={clsx(
        "text-left text-sm leading-normal font-normal text-neutral-500 dark:text-neutral-400 group-has-data-horizontal/field:text-balance [[data-variant=legend]+&]:-mt-1.5",
        "last:mt-0 nth-last-2:-mt-1",
        "[&>a]:underline [&>a]:underline-offset-4 [&>a:hover]:text-neutral-900 dark:[&>a:hover]:text-neutral-200",
        local.class,
      )}
      {...others}
    />
  );
};
