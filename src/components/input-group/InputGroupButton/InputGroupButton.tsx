import { Button } from "~/components/button/Button";
import type { InputGroupButtonProps } from "./InputGroupButton.types";
import { createMemo, mergeProps, splitProps } from "solid-js";
import { clsx } from "~/utils";
import { inputGroupButtonVariants } from "./InputGroupButton.styles";

export const InputGroupButton = (props: InputGroupButtonProps) => {
  const merged = mergeProps(
    { type: "button", variant: "ghost", size: "xs" } as const,
    props,
  );
  const [local, others] = splitProps(merged, [
    "size",
    "variant",
    "class",
    "classList",
  ]);

  const iconOnly = createMemo(
    () =>
      (others.icon &&
        typeof others.icon === "object" &&
        "ariaLabel" in others.icon &&
        "icon" in others.icon) ||
      // @ts-expect-error
      (!!others.icon && !others.children),
  );

  return (
    // @ts-expect-error
    <Button
      {...others}
      data-size={local.size}
      variant={local.variant}
      class={clsx(
        inputGroupButtonVariants({
          size: iconOnly() ? undefined : local.size,
          iconSize: iconOnly() ? local.size : undefined,
        }),
        local.class,
      )}
    />
  );
};
