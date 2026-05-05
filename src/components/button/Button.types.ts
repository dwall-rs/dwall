import type { JSXElement } from "solid-js";
import type {
  BaseProps,
  MakeRequired,
  MouseEventHandler,
  PolymorphicProps,
} from "~/types";

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
}

export type ButtonIconProps =
  | {
      /** Button content */
      children: JSXElement;

      icon?: JSXElement;

      iconPosition?: "left" | "right";
    }
  | { icon: { icon: JSXElement; ariaLabel: string } };

export type ButtonValidElement = "button" | "a";

export type ButtonProps<T extends ButtonValidElement> = Omit<
  PolymorphicProps<T, BaseButtonProps & { onClick?: MouseEventHandler<T> }>,
  "children"
> &
  ButtonIconProps;

export type ResolvedBaseButtonProps = {
  /** Button content */
  children?: JSXElement;

  icon?: JSXElement | { icon: JSXElement; ariaLabel: string };

  iconPosition?: "left" | "right";
};

export type ResolvedButtonProps<T extends ButtonValidElement> =
  PolymorphicProps<T, BaseButtonProps & { onClick?: MouseEventHandler<T> }> &
    ResolvedBaseButtonProps;

export type DefaultStyleProps = MakeRequired<
  BaseButtonProps,
  "size" | "variant"
> & { as: ButtonValidElement };

export type ButtonPropsWithDefault<T extends ButtonValidElement> =
  PolymorphicProps<T, BaseButtonProps & { onClick?: MouseEventHandler<T> }>;
