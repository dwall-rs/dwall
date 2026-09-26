import { splitProps, type JSX } from "solid-js";
import { clsx } from "~/utils";
import { callEventHandler } from "../call-event-handler";
import {
  createToggleGroupState,
  ToggleGroupContext,
} from "./ToggleGroup.context";
import { toggleGroupVariants } from "./ToggleGroup.styles";
import type { ToggleGroupProps, ToggleGroupValue } from "./ToggleGroup.types";
import { useToggleGroupKeyboard } from "./useToggleGroupKeyboard";

/**
 * ToggleGroup 根组件:渲染 `<div role="group">`,输出样式与语义钩子
 * (`data-orientation` / `data-vertical` / `data-horizontal` / `data-multiple`
 * / `data-disabled` / `data-spacing` / `data-variant` / `data-size`),
 * 并挂载 roving focus 键盘导航。
 *
 * 单选模式(默认)的 value 是标量,多选模式(`multiple`)才是数组:
 *
 * @example
 * ```tsx
 * // 单选
 * <ToggleGroup defaultValue="bold" onValueChange={(v) => ...}>
 *   <ToggleGroupItem value="bold">Bold</ToggleGroupItem>
 * </ToggleGroup>
 *
 * // 多选
 * <ToggleGroup multiple value={["bold"]} onValueChange={(v) => ...}>
 *   <ToggleGroupItem value="bold">Bold</ToggleGroupItem>
 * </ToggleGroup>
 * ```
 */
export function ToggleGroup<TValue extends ToggleGroupValue = ToggleGroupValue>(
  props: ToggleGroupProps<TValue>,
): JSX.Element {
  const [local, rest] = splitProps(props, [
    "value",
    "defaultValue",
    "onValueChange",
    "multiple",
    "disabled",
    "orientation",
    "loopFocus",
    "spacing",
    "variant",
    "size",
    "class",
    "classList",
    "style",
    "dir",
    "onKeyDown",
    "children",
  ]);

  // 默认值与单选/多选的差异统一由状态层处理,渲染用 ctx 里的已解析值
  const ctx = createToggleGroupState<TValue>(local);
  const { handleKeyDown } = useToggleGroupKeyboard(ctx);
  const isVertical = () => ctx.orientation() === "vertical";

  return (
    <div
      role="group"
      data-slot="toggle-group"
      data-variant={ctx.variant()}
      data-size={ctx.size()}
      data-spacing={ctx.spacing()}
      data-orientation={ctx.orientation()}
      // 拼接态样式依赖 data-vertical / data-horizontal(与 tabs 根保持一致)
      data-vertical={isVertical() ? "" : null}
      data-horizontal={isVertical() ? null : ""}
      data-multiple={ctx.multiple() ? "" : null}
      data-disabled={ctx.disabled() ? "" : null}
      aria-disabled={ctx.disabled() ? "true" : undefined}
      dir={local.dir}
      // --gap 作为 spacing 的样式入口,用户 style 可覆盖
      style={{ "--gap": ctx.spacing(), ...local.style } as JSX.CSSProperties}
      class={clsx(toggleGroupVariants(), local.class)}
      classList={local.classList}
      onKeyDown={(e) => {
        callEventHandler(local.onKeyDown, e);
        if (!e.defaultPrevented) handleKeyDown(e);
      }}
      {...rest}
    >
      <ToggleGroupContext.Provider value={ctx}>
        {local.children}
      </ToggleGroupContext.Provider>
    </div>
  );
}
