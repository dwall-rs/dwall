import type { VariantProps } from "class-variance-authority";
import type { ComponentProps } from "solid-js";
import type { toggleVariants } from "~/components/ui/toggle/Toggle/Toggle.styles";
import type { HTMLAttributes, PolymorphicProps } from "~/types";
import type { ToggleGroupValue } from "../ToggleGroup/ToggleGroup.types";

/**
 * 必须先从 button 属性里摘掉原生的 `value`(Solid 声明为 `string | undefined`),
 * 否则会与下面的 `value: TValue` 求交叉:`number & string` 会塌缩成 `never`。
 */
type BaseToggleGroupItemProps<TValue extends ToggleGroupValue> = VariantProps<
  typeof toggleVariants
> &
  Omit<HTMLAttributes<"button">, "value"> & {
    /** 当前 item 的唯一值,类型由泛型参数决定(与所属 ToggleGroup 保持一致) */
    value: TValue;
    /** 组件内部会先处理选中,再调用用户回调(与 tabs 的 trigger 一致) */
    onClick?: ComponentProps<"button">["onClick"];
    /** 聚焦即更新 roving focus 高亮,用户回调会被一并调用 */
    onFocus?: ComponentProps<"button">["onFocus"];
  };

/**
 * 泛型参数约束 `value` 的类型(默认 `string | number`)。
 * 子组件无法从父级 `ToggleGroup` 推导类型,因此这里由 `value` 自身推导,
 * 或显式指定:`<ToggleGroupItem<"bold" | "italic"> value="bold" />`。
 */
export type ToggleGroupItemProps<
  TValue extends ToggleGroupValue = ToggleGroupValue,
> = PolymorphicProps<"button", BaseToggleGroupItemProps<TValue>, false>;
