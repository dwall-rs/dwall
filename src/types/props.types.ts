import type { ComponentProps, JSX, ValidComponent } from "solid-js";
import type { DynamicProps } from "solid-js/web";

export type EmptyObject = Record<never, never>;

export type MouseEventHandler<T extends keyof HTMLElementTagNameMap> = (
  e: MouseEvent & {
    currentTarget: HTMLElementTagNameMap[T];
    target: Element;
  },
) => void;

export type TouchEventHandler<T extends keyof HTMLElementTagNameMap> = (
  e: TouchEvent & {
    currentTarget: HTMLElementTagNameMap[T];
    target: Element;
  },
) => void;

export interface BaseProps {
  class?: string;
  classList?: Record<string, boolean | undefined>;
  dir?: "ltr" | "rtl" | "auto";
  style?: JSX.CSSProperties;
}

export type NonNullableProps<T, K extends keyof T> = Omit<T, K> & {
  [P in K]-?: Exclude<T[P], null | undefined>;
};

/**
 * 将任意含 `component` 属性的多态 Props 解析为内部确定类型
 *
 * - 若 TProps 含 `component?:T`，则展开 T 对应的 ComponentProps
 * - 否则回退到 TDefault 的 ComponentProps
 */
export type ResolvedProps<TProps, TDefault extends ValidComponent> = (
  "component" extends keyof TProps
    ? NonNullable<TProps["component"]>
    : TDefault
) extends infer TComp
  ? TComp extends ValidComponent
    ? Omit<TProps, "component"> &
        ComponentProps<TComp> & { component?: ValidComponent }
    : TProps & { component?: ValidComponent }
  : never;

export type HTMLAttributes<T extends ValidComponent> = Omit<
  ComponentProps<T>,
  keyof BaseProps
> &
  BaseProps;

type PolymorphicProp<
  T extends ValidComponent,
  Enable extends boolean,
> = Enable extends true ? { component?: T } : EmptyObject;

export type PolymorphicProps<
  T extends ValidComponent,
  O = EmptyObject,
  EnableAs extends boolean = true,
> = O &
  Omit<DynamicProps<T>, keyof O | "component"> &
  PolymorphicProp<T, EnableAs>;
