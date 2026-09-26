import type { Alignment, Side } from "~/lib";
import type { BaseProps, PolymorphicProps } from "~/types";

export type TooltipSide = Side | "inline-start" | "inline-end";

interface BaseTooltipContentProps extends BaseProps {
  /** 贴 trigger 的哪一侧，默认 'top'。支持 Base UI 的 logical side。 */
  side?: TooltipSide;
  /** 沿边对齐方式，默认 'center'（不偏移，居中对齐） */
  align?: Alignment | "center";
  /** 沿主轴（side 指向的方向）与 trigger 之间的间距（像素），默认 8 */
  sideOffset?: number;
  /**
   * 沿交叉轴（对齐方向）的偏移（像素），默认 0——align 是 'start'/'end' 时
   * 微调对齐位置用。底层直接对应 offset middleware 的 crossAxis。
   */
  alignOffset?: number;
  /** 距视口边缘的最小间距（像素），用于 flip/shift 的边界检测，默认 8 */
  collisionPadding?: number;
}

export type TooltipContentProps = PolymorphicProps<
  "div",
  BaseTooltipContentProps,
  false
>;
