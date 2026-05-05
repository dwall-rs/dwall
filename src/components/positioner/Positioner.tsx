import { mergeProps, Show, splitProps } from "solid-js";
import type { PositionerProps } from "./Positioner.types";
import { Portal } from "solid-js/web";
import { clsx } from "~/utils";
import { usePositioner } from "./usePositioner";

/**
 * 通用 `position: fixed` 浮层定位组件。
 *
 * ## 普通模式
 * 浮层整体相对 trigger 定位。
 *
 * ```tsx
 * <Positioner triggerRef={ref} side="bottom" align="start" open={open()}>
 *   <Tooltip />
 * </Positioner>
 * ```
 *
 * ## 锚点对齐模式（anchorRef）
 * 浮层内的子元素（锚点）与 trigger 精确对齐，整体位置由此反推。
 *
 * ```tsx
 * <Positioner
 *   triggerRef={ref}
 *   anchorRef={activeItemEl()}
 *   anchorSide="top"
 *   anchorAlign="start"
 *   open={open()}
 * />
 * ```
 *
 * ## 边界检测
 * 默认自动启用 flip → shift → size 三级策略，检测最近滚动容器与 viewport 的交集边界。
 *
 * ```tsx
 * // 只允许 flip + shift，不裁剪尺寸
 * <Positioner overflowStrategy={["flip", "shift"]} ... />
 *
 * // 自定义边界元素
 * <Positioner boundary={containerEl} boundaryPadding={12} ... />
 *
 * // 禁用边界检测
 * <Positioner overflowStrategy={[]} ... />
 * ```
 */
export const Positioner = (props: PositionerProps) => {
  const merged = mergeProps(
    {
      side: "bottom",
      sideOffset: 4,
      align: "start",
      alignOffset: 0,
      alignItemWithTrigger: false,
      anchorSide: "top",
      anchorAlign: "start",
      boundaryPadding: 8,
      overflowStrategy: ["flip", "shift", "size"] as Array<
        "flip" | "shift" | "size"
      >,
      open: true,
    } as const,
    props,
  );

  const [local, others] = splitProps(merged, [
    "side",
    "sideOffset",
    "align",
    "alignOffset",
    "alignItemWithTrigger",
    "anchorSide",
    "anchorAlign",
    "boundaryPadding",
    "overflowStrategy",
    "triggerRef",
    "anchorRef",
    "boundary",
    "style",
    "open",
    "class",
  ]);

  const { ref, style } = usePositioner({
    get props() {
      return local;
    },
  });

  return (
    <Show when={local.open}>
      <Portal>
        <div
          ref={ref}
          class={clsx("fixed", local.class)}
          style={style()}
          data-side={local.side}
          data-align={local.align}
          aria-hidden={!local.open}
          {...others}
        />
      </Portal>
    </Show>
  );
};
