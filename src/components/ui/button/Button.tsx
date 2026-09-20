import {
  children,
  createMemo,
  createUniqueId,
  type JSXElement,
  mergeProps,
  splitProps,
  type ValidComponent,
} from "solid-js";
import { Dynamic } from "solid-js/web";
import { clsx, logger } from "~/utils";
import s from "./Button.styles";
import type { ButtonProps, ButtonValidElement } from "./Button.types";

const log = logger.child("Button");

export const Button = <T extends ButtonValidElement = "button">(
  props: ButtonProps<T>,
) => {
  const merged = mergeProps(
    {
      variant: "primary",
      size: "md",
      component: "button",
      id: createUniqueId(),
    } as const,
    props,
  );

  const [local, others] = splitProps(merged, [
    "component",
    "variant",
    "size",
    "icon",
    "iconPosition",
    "class",
    "classList",
    "children",
    "onClick",
  ]);

  const iconOnly = createMemo(() => !!local.icon && !local.children);

  const iconWithAttr = (
    icon: JSXElement,
    position: "inline-start" | "inline-end",
  ) => <span data-icon={position}>{icon}</span>;

  const resolved = children(() => {
    if (import.meta.env.DEV && iconOnly() && !others["aria-label"]) {
      log.warn("icon-only button must have an aria-label");
    }

    return local.icon ? (
      iconOnly() ? (
        <>
          {local.icon}
          <span class="sr-only">{others["aria-label"]}</span>
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
    );
  });

  return (
    <Dynamic
      data-slot="button"
      component={local.component as ValidComponent}
      type={
        local.component === "button" ? (others.type ?? "button") : undefined
      }
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
