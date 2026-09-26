import type { ParentProps } from "solid-js";

/**
 * <TooltipGroup> 是可选的：把多个 <Tooltip> 包在一起，组内相邻 trigger 之间
 * 切换 hover 时，会跳过 openDelay 并触发"从旧位置滑到新位置"的过渡效果，
 * 而不是旧的先走完退场动画、新的再等 openDelay 出现。
 *
 * 不需要这个效果的话，<Tooltip> 完全可以脱离 <TooltipGroup> 单独使用，
 * 行为不受影响（内部用 useTooltipGroupContext 判断是否存在，缺失时静默降级）。
 */
export type TooltipGroupProps = ParentProps;
