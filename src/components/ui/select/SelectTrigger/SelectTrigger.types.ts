import type { ParentProps } from "solid-js";

export interface SelectTriggerProps extends ParentProps {
  /**
   * 渲染成什么标签，默认 "button"。传 "g"/"div" 等可以适配非按钮场景，
   * 内部用 Dynamic 渲染，动态标签会按需要走对应的命名空间创建。
   */
  as?: string;
  class?: string;
  disabled?: boolean;
  /**
   * 转发给最终渲染元素的 ref。类型和 Solid 的 ref 约定一致（赋值式/回调式都可以，
   * 编译器会统一规整成回调函数再转发进来，细节见
   * https://docs.solidjs.com/concepts/refs#forwarding-refs）。
   */
  ref?: Element | ((el: Element) => void);
  /**
   * 其余任意原生属性/事件都会原样透传到最终渲染的元素上——最常见的场景是
   * `onClick`。内部用来开关下拉的监听器走的是 addEventListener，直接绑在
   * DOM 节点上，和这里透传的 JSX onXxx prop 是完全独立的两套机制，不会互相覆盖。
   */
  [key: string]: any;
}
