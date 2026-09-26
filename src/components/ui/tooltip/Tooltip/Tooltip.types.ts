import type { Accessor, ParentProps } from "solid-js";
import type { Rect } from "~/lib";

export interface TooltipProps extends ParentProps {
  /** 受控 open 状态；不传则内部自管理（非受控模式） */
  open?: boolean;
  /** 非受控模式下的初始状态，默认 false */
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  /**
   * 鼠标悬停多久后打开（ms）。键盘 focus 打开不受这个延迟影响——
   * 无障碍角度看，键盘用户不应该被迫等待，所以 focus 走 openImmediate。
   * 默认 700，贴近系统 tooltip 的习惯延迟。
   */
  openDelay?: number;
  /**
   * 鼠标移出后多久关闭（ms），留出时间让用户把鼠标移动到 tooltip 内容上
   * （比如内容里有可以选中复制的文字）。默认 150。
   */
  closeDelay?: number;
  /** 禁用整个 tooltip（trigger 上也可以单独再禁用） */
  disabled?: boolean;
}

export interface TooltipContextValue {
  open: Accessor<boolean>;
  disabled: Accessor<boolean>;
  /** 用于 trigger 的 aria-describedby 和 content 的 id 关联 */
  contentId: string;
  reference: Accessor<Element | undefined>;
  setReference: (el: Element) => void;
  floating: Accessor<HTMLElement | undefined>;
  setFloating: (el: HTMLElement | undefined) => void;
  /** 按 openDelay 延迟打开；已经打开则不做任何事，只清掉可能存在的关闭计时器 */
  requestOpen: () => void;
  /** 按 closeDelay 延迟关闭 */
  requestClose: () => void;
  /** 跳过延迟立即打开（键盘 focus 场景） */
  openImmediate: () => void;
  /** 跳过延迟立即关闭（Escape / blur 场景） */
  closeImmediate: () => void;
  /** 取消任何待执行的关闭计时器，但不改变当前状态（鼠标移到 content 上时用） */
  keepOpen: () => void;
  /**
   * 命中 <TooltipGroup> 的"抢占"时，记录被抢占的旧 tooltip 此刻的矩形——
   * TooltipContent 用它计算出一个初始偏移量，做"从旧位置滑到新位置"的过渡，
   * 消费一次之后应该调用 clearPendingSlideFrom 清空，避免后续 autoUpdate
   * 重新定位（比如滚动、窗口 resize）时又误触发一次滑入动画。
   */
  pendingSlideFrom: Accessor<Rect | undefined>;
  clearPendingSlideFrom: () => void;
  /**
   * 被 <TooltipGroup> 抢占强制关闭时会置为 true——TooltipContent 的 Presence
   * 逻辑据此判断这次关闭要不要跳过退场动画、立即卸载（避免和正在滑入的新
   * tooltip 同时出现在屏幕上）。consumeSuppressExitAnimation 读取的同时会
   * 把它重置为 false，只消费一次，不影响后续正常关闭时的退场动画。
   */
  suppressExitAnimation: Accessor<boolean>;
  consumeSuppressExitAnimation: () => boolean;
}
