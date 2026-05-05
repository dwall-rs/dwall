import {
  children,
  createMemo,
  createUniqueId,
  type JSXElement,
  mergeProps,
  splitProps,
} from "solid-js";
import { Dynamic } from "solid-js/web";
import { clsx } from "~/utils/clsx";
import s from "./Button.styles";
import type {
  ButtonProps,
  ButtonValidElement,
  DefaultStyleProps,
  ResolvedButtonProps,
} from "./Button.types";

export const Button = <T extends ButtonValidElement = "button">(
  props: ButtonProps<T>,
) => {
  const merged = mergeProps(
    {
      variant: "primary",
      size: "md",
      as: "button",
      id: createUniqueId(),
    } satisfies Partial<DefaultStyleProps>,
    props as ResolvedButtonProps<T>,
  );

  const [local, others] = splitProps(merged, [
    "as",
    "variant",
    "size",
    "icon",
    "iconPosition",
    "class",
    "classList",
    "children",
    "onClick",
  ]);

  const iconOnly = createMemo(
    () =>
      (local.icon &&
        typeof local.icon === "object" &&
        "ariaLabel" in local.icon &&
        "icon" in local.icon) ||
      (!!local.icon && !local.children),
  );

  const iconWithAttr = (
    icon: JSXElement,
    position: "inline-start" | "inline-end",
  ) => <span data-icon={position}>{icon}</span>;

  const resolved = children(() =>
    local.icon ? (
      typeof local.icon === "object" &&
      "ariaLabel" in local.icon &&
      "icon" in local.icon ? (
        <>
          {local.icon.icon}
          <span class="sr-only">{local.icon.ariaLabel}</span>
        </>
      ) : (
        <>
          {(!local.iconPosition || local.iconPosition === "left") &&
            iconWithAttr(local.icon, "inline-start")}
          {local.children}
          {local.iconPosition === "right" &&
            iconWithAttr(local.icon, "inline-end")}
        </>
      )
    ) : (
      local.children
    ),
  );

  return (
    // @ts-expect-error
    <Dynamic
      data-slot="button"
      component={local.as}
      type={local.as === "button" ? (others.type ?? "button") : undefined}
      onClick={local.onClick}
      class={clsx(
        s({
          variant: local.variant,
          size: iconOnly() ? undefined : local.size,
          iconSize: iconOnly() ? local.size : undefined,
        }),
        local.class,
      )}
      {...others}
    >
      {resolved()}
    </Dynamic>
  );
};
