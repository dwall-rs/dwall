import type { ParentProps } from "solid-js";
import type { Placement } from "~/lib";

export interface SelectContentProps extends ParentProps {
  /**
   * 还没有选中值时（占位符状态）的首选摆放位置，默认 'bottom-start'。
   * 一旦有选中值，定位策略会切换成"选中项对齐"（细节见 SelectContent.utils.ts
   * 里的 alignSelectedItem），这个 prop 在那种情况下不生效——这也是 SelectContent
   * 没有跟 TooltipContent 一样改成 side/align 两个独立 prop 的原因：
   * "选中项对齐"这套定位模型本身就没有 side 的概念（面板不是贴在 trigger
   * 的某一侧，而是让选中项本身盖在 trigger 上），用 side/align 强行套用
   * 反而不准确。
   */
  placement?: Placement;
  /** 距视口边缘的最小间距（像素），用于 flip/shift 的边界检测，默认 8 */
  collisionPadding?: number;
  class?: string;
  /**
   * 其余任意原生 div 属性都会原样透传到最终渲染的元素上——和 SelectTrigger
   * 是同一套约定。
   */
  [key: string]: any;
}
