import { mergeProps, onCleanup, onMount, splitProps, type JSX } from "solid-js";
import { clsx } from "~/utils";
import { toggleVariants } from "~/components/ui/toggle/Toggle/Toggle.styles";
import { callEventHandler } from "../call-event-handler";
import { useToggleGroupContext } from "../ToggleGroup/ToggleGroup.context";
import type { ToggleGroupValue } from "../ToggleGroup/ToggleGroup.types";
import { toggleGroupItemVariants } from "./ToggleGroupItem.styles";
import type { ToggleGroupItemProps } from "./ToggleGroupItem.types";

/**
 * ToggleGroupItem:ToggleGroup 内的两态按钮。
 *
 * - 选中值由 ToggleGroup 统一持有,点击通过 `ctx.toggleItem` 触发变更
 * - disabled 继承整组状态,可在 item 上单独禁用
 * - roving tabindex:整组只有一个 tab 停靠点,方向键在同组内移动焦点
 */
export function ToggleGroupItem<
  TValue extends ToggleGroupValue = ToggleGroupValue,
>(props: ToggleGroupItemProps<TValue>): JSX.Element {
  // context 用宽值类型读取:item 无法从父级推导 group 的值类型,
  // 自己的 TValue 始终可以传给宽类型参数
  const ctx = useToggleGroupContext("ToggleGroupItem");
  const merged = mergeProps({ type: "button" as const }, props);

  const [local, rest] = splitProps(merged, [
    "value",
    "variant",
    "size",
    "class",
    "classList",
    "disabled",
    "type",
    "onClick",
    "onFocus",
    "children",
  ]);

  let elementRef: HTMLButtonElement | undefined;

  // 组级未显式指定 variant/size 时,item 可单独覆盖
  const variant = () => ctx.variant() ?? local.variant ?? "default";
  const size = () => ctx.size() ?? local.size ?? "default";
  const pressed = () => ctx.isPressed(local.value);
  const disabled = () => ctx.disabled() || !!local.disabled;

  // roving tabindex:高亮 item 持 0;高亮缺失或指向不可用 item(disabled/卸载)时,
  // 首个可用 item 持 0 作为键盘起点,避免整组键盘不可达
  const tabIndex = () => {
    if (disabled()) return -1;
    const highlighted = ctx.highlightedValue();
    if (highlighted === local.value && ctx.isUsable(local.value)) return 0;
    if (
      (highlighted === undefined || !ctx.isUsable(highlighted)) &&
      ctx.isFirstEnabled(local.value)
    ) {
      return 0;
    }
    return -1;
  };

  onMount(() => {
    onCleanup(
      ctx.registerItem({
        value: local.value,
        disabled,
        element: elementRef!,
      }),
    );
  });

  return (
    <button
      ref={(el) => {
        elementRef = el;
      }}
      type={local.type}
      disabled={disabled()}
      tabindex={tabIndex()}
      data-slot="toggle-group-item"
      data-variant={variant()}
      data-size={size()}
      data-spacing={ctx.spacing()}
      data-pressed={pressed() ? "" : null}
      data-disabled={disabled() ? "" : null}
      data-state={pressed() ? "on" : "off"}
      aria-pressed={pressed()}
      class={clsx(
        toggleVariants({ variant: variant(), size: size() }),
        toggleGroupItemVariants(),
        local.class,
      )}
      classList={local.classList}
      onClick={(e) => {
        // 用户回调优先,可用 preventDefault() 阻止本次选中
        callEventHandler(local.onClick, e);
        if (!e.defaultPrevented && !disabled()) {
          ctx.toggleItem(local.value, e, e.currentTarget);
        }
      }}
      onFocus={(e) => {
        // 聚焦更新高亮(roving focus 起点)
        if (!disabled()) ctx.setHighlightedValue(local.value);
        callEventHandler(local.onFocus, e);
      }}
      {...rest}
    >
      {local.children}
    </button>
  );
}
