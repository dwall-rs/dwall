import type { JSX, ComponentProps, ValidComponent } from "solid-js";

export type EmptyObject = Record<never, never>;

export type MouseEventHandler<T extends keyof HTMLElementTagNameMap> = (
  e?: MouseEvent & {
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

type AsProp<
  T extends ValidComponent,
  Enable extends boolean,
> = Enable extends true ? { as?: T } : EmptyObject;

export type PolymorphicProps<
  T extends ValidComponent,
  P = EmptyObject,
  EnableAs extends boolean = true,
> = P & Omit<ComponentProps<T>, keyof P> & AsProp<T, EnableAs>;

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
 * 将任意含 `as` 属性的多态 Props 解析为内部确定类型
 *
 * - 若 TProps 含 `as?: T`，则展开 T 对应的 ComponentProps
 * - 否则回退到 TDefault 的 ComponentProps
 */
export type ResolvedProps<TProps, TDefault extends ValidComponent> = (
  "as" extends keyof TProps
    ? NonNullable<TProps["as"]>
    : TDefault
) extends infer TComp
  ? TComp extends ValidComponent
    ? Omit<TProps, "as"> & ComponentProps<TComp> & { as?: ValidComponent }
    : TProps & { as?: ValidComponent }
  : never;
