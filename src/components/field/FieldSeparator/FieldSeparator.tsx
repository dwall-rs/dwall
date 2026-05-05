import { Show, splitProps } from "solid-js";
import { clsx } from "~/utils";
import type { FieldSeparatorProps } from "./FieldSeparator.types";
import { Separator } from "~/components/separator";

export const FieldSeparator = (props: FieldSeparatorProps) => {
  const [local, others] = splitProps(props, ["class", "classList", "children"]);

  return (
    <div
      data-slot="field-separator"
      data-content={!!local.children}
      class={clsx(
        "relative -my-2 h-5 text-sm group-data-[variant=outline]/field-group:-mb-2",
        local.class,
      )}
      {...others}
    >
      <Separator class="absolute inset-0 top-1/2" />

      <Show when={local.children}>
        <span
          class={clsx(
            "relative mx-auto block w-fit px-2",
            "bg-white text-neutral-500",
            "dark:bg-neutral-950 dark:text-neutral-400",
          )}
          data-slot="field-separator-content"
        >
          {local.children}
        </span>
      </Show>
    </div>
  );
};
