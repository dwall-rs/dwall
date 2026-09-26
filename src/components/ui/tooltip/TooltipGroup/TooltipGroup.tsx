import { TooltipGroupContext } from "./TooltipGroup.context";
import type {
  ActiveTooltipEntry,
  TooltipGroupContextValue,
} from "./TooltipGroup.context";
import type { TooltipGroupProps } from "./TooltipGroup.types";

export function TooltipGroup(props: TooltipGroupProps) {
  // 用一个模块外可变量而不是 Solid signal 存"当前活跃条目"是有意的：
  // 这纯粹是组件间协调用的内部状态，不需要驱动任何渲染，用 signal 反而会
  // 引入不必要的响应式开销和多余的重渲染触发。
  let activeEntry: ActiveTooltipEntry | undefined;

  const registerActive: TooltipGroupContextValue["registerActive"] = (
    entry,
  ) => {
    activeEntry = entry;
    return () => {
      // 只清自己注册的那条，避免"A 还没来得及 unregister，B 已经抢注册"时，
      // A 的 cleanup 误把 B 刚设置的 activeEntry 清掉。
      if (activeEntry === entry) activeEntry = undefined;
    };
  };

  const claim: TooltipGroupContextValue["claim"] = () => {
    if (!activeEntry) return undefined;
    const rect = activeEntry.rect();
    activeEntry.forceClose();
    activeEntry = undefined;

    // 防御性检查：一个真实可见的 tooltip 不可能宽高都是 0——如果读到的是
    // 这种全 0 矩形，大概率是读到了一个已经从 DOM 里摘除的 detached 元素
    // （getBoundingClientRect 规范规定这种情况下就是全 0）。这种情况下
    // 宁可放弃这次平移过渡（新 tooltip 走正常的 openDelay 流程），也不要
    // 把 (0,0) 当成合法的滑入起点，出现"从左上角滑入"这种明显错误的观感。
    if (!rect || (rect.width === 0 && rect.height === 0)) return undefined;
    return rect;
  };

  const ctx: TooltipGroupContextValue = { registerActive, claim };

  return (
    <TooltipGroupContext.Provider value={ctx}>
      {props.children}
    </TooltipGroupContext.Provider>
  );
}
