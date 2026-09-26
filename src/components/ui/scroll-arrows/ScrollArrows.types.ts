import type { Accessor } from "solid-js";

export interface ScrollArrowsProps {
  /** 滚动容器 accessor；组件内部据此计算上下边缘状态 */
  target: Accessor<HTMLElement | undefined>;
  /** 两个箭头共用的额外 class */
  class?: string;
  /** 顶部箭头额外 class(圆角等) */
  upClass?: string;
  /** 底部箭头额外 class(圆角等) */
  downClass?: string;
}

export interface ScrollArrowButtonProps {
  direction: "up" | "down";
  visible: boolean;
  target: Accessor<HTMLElement | undefined>;
  class?: string;
}
