import { createContext, useContext } from "solid-js";
import type { Rect } from "~/lib";

export interface ActiveTooltipEntry {
  /** 读取这个 tooltip 浮层此刻的实时矩形，供下一个抢占它的 tooltip 当"滑入起点"用 */
  rect: () => Rect | undefined;
  /** 立即关闭（跳过 closeDelay 和退场动画）——组内切换到别的 trigger 时调用 */
  forceClose: () => void;
}

export interface TooltipGroupContextValue {
  /** 打开时注册自己为"组内当前活跃的 tooltip"，返回取消注册函数（关闭/卸载时调用） */
  registerActive: (entry: ActiveTooltipEntry) => () => void;
  /**
   * 组内某个 tooltip 即将打开时调用：
   * 如果组里当前已经有一个活跃的 tooltip，立即强制关闭它、拿到它此刻的矩形
   * 并返回（用作平移过渡的起点）；如果组内当前没有任何活跃 tooltip，返回
   * undefined，调用方应该走正常的 openDelay 流程。
   *
   * 用"是否存在活跃 tooltip"而不是"上一个关闭后多久之内"这种时间窗口来判断，
   * 是为了精确匹配"旧的还在退场倒计时里、这时候 hover 了别的 trigger"这个场景——
   * 这种情况下旧的严格来说还没真正关闭（open() 依然是 true），基于时间窗口的
   * 判断会因为"还没关闭、没触发通知"而错过这次协调，直接查"当前活跃"更可靠。
   */
  claim: () => Rect | undefined;
}

export const TooltipGroupContext = createContext<TooltipGroupContextValue>();

/**
 * 注意：和其他 useXxxContext 不同，这里缺失 context 时不抛错——
 * <TooltipGroup> 是可选的，单独使用 <Tooltip> 不需要包一层。
 */
export function useTooltipGroupContext(): TooltipGroupContextValue | undefined {
  return useContext(TooltipGroupContext);
}
