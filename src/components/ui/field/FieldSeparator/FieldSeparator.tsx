import { Show, splitProps } from "solid-js";
import { clsx } from "~/utils";
import type { FieldSeparatorProps } from "./FieldSeparator.types";
import { Separator } from "~/components/ui/separator";

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
          class="relative mx-auto block w-fit bg-background px-2 text-muted-foreground"
          data-slot="field-separator-content"
        >
          {local.children}
        </span>
      </Show>
    </div>
  );
};
