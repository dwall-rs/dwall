import { splitProps } from "solid-js";
import type { LabelProps } from "./Label.types";
import { clsx } from "~/utils";

export const Label = (props: LabelProps) => {
  let ref: HTMLLabelElement | undefined;
  const [local, others] = splitProps(props, ["for", "class", "classList"]);

  const handleClick = (e: MouseEvent) => {
    const htmlFor = local.for;
    if (htmlFor) {
      const target = document.querySelector<HTMLElement>(
        `[aria-labelledby="${htmlFor}"]`,
      );
      if (!target) return;

      target.click();

      e.preventDefault();
      return;
    }

    const target = ref?.querySelector<HTMLElement>("[aria-labelledby]");
    if (target && !target.contains(e.target as Node)) {
      e.preventDefault();
      target.click();
    }
  };

  return (
    // biome-ignore lint/a11y/noLabelWithoutControl: ignore
    <label
      ref={ref}
      for={local.for}
      data-slot="label"
      class={clsx(
        "flex items-center gap-2 text-sm leading-none font-medium select-none group-data-[disabled=true]:pointer-events-none group-data-[disabled=true]:opacity-50 peer-disabled:cursor-not-allowed peer-disabled:opacity-50",
        local.class,
      )}
      classList={local.classList}
      onClick={handleClick}
      {...others}
    />
  );
};
