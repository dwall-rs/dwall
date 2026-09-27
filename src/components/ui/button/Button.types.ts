import type { JSXElement } from "solid-js";
import type { BaseProps, MouseEventHandler, PolymorphicProps } from "~/types";

export type ButtonVariant =
  | "primary"
  | "outline"
  | "secondary"
  | "ghost"
  | "destructive"
  | "link";

export type ButtonSize = "md" | "xs" | "sm" | "lg";

export interface BaseButtonProps extends BaseProps {
  variant?: ButtonVariant;

  size?: ButtonSize;

  id?: string;

  children?: JSXElement;

  icon?: JSXElement;

  iconPosition?: "left" | "right";
}

export type ButtonValidElement = "button" | "a";

export type ButtonProps<T extends ButtonValidElement = "button"> =
  PolymorphicProps<T, BaseButtonProps & { onClick?: MouseEventHandler<T> }>;
