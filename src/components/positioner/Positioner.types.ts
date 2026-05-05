import type { JSX } from "solid-js";
import type { PolymorphicProps } from "~/types";

export interface BasePositionerProps {
  // ── 基础定位 ────────────────────────────────────────────────────────────────

  /** 浮层出现在 trigger 的哪一侧 */
  side?: "top" | "bottom" | "left" | "right";
  /** 浮层与 trigger 在主轴方向上的间距（px） */
  sideOffset?: number;
  /** 浮层在交叉轴上的对齐方式 */
  align?: "start" | "center" | "end";
  /** 交叉轴方向的额外偏移（px） */
  alignOffset?: number;
  /** 浮层交叉轴尺寸是否拉伸至与 trigger 一致 */
  alignItemWithTrigger?: boolean;

  // ── 锚点对齐 ────────────────────────────────────────────────────────────────

  /**
   * 浮层内需要与 trigger 对齐的子元素引用。
   * 启用后 Positioner 会移动整个浮层，使锚点元素的位置与 trigger 对齐。
   */
  anchorRef?: HTMLElement | undefined;
  /** 锚点的哪条边与 trigger 的同名边对齐（默认 "top"） */
  anchorSide?: "top" | "bottom" | "left" | "right";
  /** 锚点在交叉轴上的对齐方式（默认 "start"） */
  anchorAlign?: "start" | "center" | "end";

  // ── 边界检测（Boundary / Overflow）─────────────────────────────────────────

  /**
   * 边界检测的 padding（px），浮层距边界的最小留白。
   * 可传单值或 `{ top, right, bottom, left }` 分别设置。
   * 默认 8px。
   */
  boundaryPadding?: number | Partial<EdgeInsets>;

  /**
   * 自定义边界元素。
   * - 不传：自动向上查找最近的滚动容器，并与 viewport 取交集。
   * - 传入 HTMLElement：以该元素的 client 区域为边界（仍与 viewport 取交集）。
   * - 传入 "viewport"：只使用 viewport，不检测滚动容器。
   */
  boundary?: HTMLElement | "viewport";

  /**
   * 超出边界后的处理策略（默认 ["flip", "shift", "size"]，按顺序应用）。
   *
   * - "flip"  : 当主轴空间不足时，将浮层翻转到对侧（bottom ↔ top / left ↔ right）
   * - "shift" : flip 后仍超出时，沿交叉轴平移浮层使其回到边界内
   * - "size"  : 在可用空间内限制浮层的 max-height / max-width
   *
   * 传空数组 `[]` 可禁用所有边界检测。
   */
  overflowStrategy?: Array<"flip" | "shift" | "size">;

  // ── 其他 ────────────────────────────────────────────────────────────────────

  /** trigger 元素 DOM 引用（必填） */
  triggerRef: HTMLElement | undefined;
  /** 是否显示浮层 */
  open?: boolean;

  style?: JSX.CSSProperties;
}

export type PositionerProps = PolymorphicProps<
  "div",
  BasePositionerProps,
  false
>;

export interface Rect {
  top: number;
  left: number;
  width: number;
  height: number;
}

export interface EdgeInsets {
  top: number;
  right: number;
  bottom: number;
  left: number;
}

export type RequiredPositionerProps = Required<
  Pick<
    PositionerProps,
    keyof Omit<
      BasePositionerProps,
      "triggerRef" | "open" | "style" | "boundary" | "anchorRef"
    >
  >
>;

export interface FloatPosition {
  top: number;
  left: number;
  width?: number;
  height?: number;
  maxWidth?: number;
  maxHeight?: number;
  /**
   * 当前 resolvedSide 方向上，从浮层放置点到裁剪边界的原始可用空间。
   *
   * 与 maxHeight / maxWidth 的区别：
   *   - max* 直接限制浮层自身尺寸（由 "size" 策略写入）
   *   - available* 仅作为 CSS 变量 --available-height / --available-width 注入，
   *     供浮层内的子元素（如滚动列表）自行用 max-height: var(--available-height) 控制，
   *     适合浮层内同时存在固定区域（搜索框、底部操作栏）和滚动区域的组合场景
   */
  availableWidth: number;
  availableHeight: number;
  /** 实际使用的 side（flip 后可能与原始 side 不同） */
  resolvedSide: "top" | "bottom" | "left" | "right";
}
