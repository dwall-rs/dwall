import { createContext, useContext, type Accessor } from "solid-js";
import type { MiddlewareData, Placement } from "~/lib";

export interface TooltipContentContextValue {
  /** createPositioner 产出的 middlewareData，TooltipArrow 从里面取 arrow 中间件算好的偏移量 */
  middlewareData: Accessor<MiddlewareData>;
  /** 触发器元素访问器，TooltipArrow 用它计算 --arrow-offset */
  reference: Accessor<Element | undefined>;
  /** 经过 flip 调整后的最终 placement，决定箭头贴哪条边、朝哪个方向 */
  placement: Accessor<Placement>;
  /** TooltipArrow 挂载时用来把自己的 DOM 节点注册给 arrow middleware（Element 而不是 HTMLElement，兼容 SVG 箭头） */
  setArrowElement: (el: Element) => void;
  /**
   * 驱动进出场动画的状态，和 TooltipContent 内层元素的 data-state 用的是
   * 同一个信号——TooltipArrow 应该用它独立播放自己的一份动画，而不是依赖
   * "作为内层元素的子节点、被动继承内层的 transform/opacity 动画"这种隐式
   * 机制（内层的 zoom-in-95 动画会给它加一个非 none 的 transform，CSS 规范
   * 规定这会让内层变成子元素新的"包含块"，和箭头故意跳过内层、以外层为
   * 基准的设计相冲突，动画播放期间会让"父子两层谁负责什么"变得不可靠）。
   */
  animationState: Accessor<"open" | "closed">;
}

export const TooltipContentContext =
  createContext<TooltipContentContextValue>();

export function useTooltipContentContext(
  component: string,
): TooltipContentContextValue {
  const ctx = useContext(TooltipContentContext);
  if (!ctx) {
    throw new Error(`<${component}> 必须渲染在 <TooltipContent> 内部`);
  }
  return ctx;
}
