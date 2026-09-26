import { createMemo } from "solid-js";
import { getAlignment, getSide } from "~/lib";
import { clsx } from "~/utils";
import { useTooltipContentContext } from "../TooltipContent/TooltipContent.context";

export interface TooltipArrowProps {
  class?: string;
}

/**
 * 箭头组件，必须渲染在 <TooltipContent> 内部。
 * 采用 shadcn Base UI 的定位方式：通过 data-side / data-align 和 CSS 变量
 * 完成贴边与对齐，不手动计算 top/left。
 */
export function TooltipArrow(props: TooltipArrowProps) {
  const ctx = useTooltipContentContext("TooltipArrow");

  const side = createMemo(() => getSide(ctx.placement()));
  const align = createMemo(() => getAlignment(ctx.placement()) ?? "center");

  const triggerWidth = () =>
    ctx.reference?.()?.getBoundingClientRect().width ?? 0;

  return (
    <div
      ref={ctx.setArrowElement}
      data-side={side()}
      data-align={align()}
      style={{ "--arrow-offset": `${triggerWidth() / 2}px` }}
      class={clsx(
        "absolute size-2.5 z-50 translate-y-[calc(-50%-2px)] rotate-45 rounded-xs bg-foreground fill-foreground",
        "data-[side=inline-end]:top-1/2! data-[side=inline-end]:-left-1 data-[side=inline-end]:-translate-y-1/2",
        "data-[side=inline-start]:top-1/2! data-[side=inline-start]:-right-1 data-[side=inline-start]:-translate-y-1/2",
        "data-[side=left]:top-1/2! data-[side=left]:right-[-0.175rem] data-[side=left]:-translate-y-1/2",
        "data-[side=right]:top-1/2! data-[side=right]:left-[-0.175rem] data-[side=right]:-translate-y-1/2",
        "data-[side=top]:-bottom-2.5 data-[side=top]:data-[align=center]:left-1/2 data-[side=top]:data-[align=center]:-translate-x-1/2 data-[side=top]:data-[align=start]:left-(--arrow-offset) data-[side=top]:data-[align=start]:-translate-x-1/2 data-[side=top]:data-[align=end]:right-(--arrow-offset) data-[side=top]:data-[align=end]:translate-x-1/2",
        "data-[side=bottom]:top-1 data-[side=bottom]:data-[align=center]:left-1/2 data-[side=bottom]:data-[align=center]:-translate-x-1/2 data-[side=bottom]:data-[align=start]:left-(--arrow-offset) data-[side=bottom]:data-[align=start]:-translate-x-1/2 data-[side=bottom]:data-[align=end]:right-(--arrow-offset) data-[side=bottom]:data-[align=end]:translate-x-1/2",
        props.class,
      )}
      aria-hidden="true"
    />
  );
}
