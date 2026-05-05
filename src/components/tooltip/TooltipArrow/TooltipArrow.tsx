import { clsx } from "~/utils";
import { useTooltipContentContext } from "../TooltipContent/TooltipContent.context";
import { useTooltipContext } from "../Tooltip/Tooltip.context";

export const TooltipArrow = () => {
  const { triggerRef } = useTooltipContext();
  const { side, align } = useTooltipContentContext();

  const triggerWidth = () => triggerRef?.offsetWidth ?? 0;

  return (
    <div
      // ref={setup}
      data-side={side}
      data-align={align}
      class={clsx(
        "absolute size-2.5 z-50 translate-y-[calc(-50%-2px)] rotate-45 rounded-xs bg-neutral-950 dark:bg-neutral-50 fill-neutral-950 dark:fill-neutral-50",
        "data-[side=inline-end]:top-1/2! data-[side=inline-end]:-left-1 data-[side=inline-end]:-translate-y-1/2 data-[side=inline-start]:top-1/2! data-[side=inline-start]:-right-1 data-[side=inline-start]:-translate-y-1/2",
        "data-[side=left]:top-1/2! data-[side=left]:right-[-0.175rem] data-[side=left]:-translate-y-1/2",
        "data-[side=right]:top-1/2! data-[side=right]:left-[-0.175rem] data-[side=right]:-translate-y-1/2",
        "data-[side=top]:-bottom-2.5 data-[side=top]:data-[align=center]:left-1/2 data-[side=top]:data-[align=center]:-translate-x-1/2 data-[side=top]:data-[align=start]:left-(--arrow-offset) data-[side=top]:data-[align=start]:-translate-x-1/2 data-[side=top]:data-[align=end]:right-(--arrow-offset) data-[side=top]:data-[align=end]:translate-x-1/2",
        "data-[side=bottom]:top-1 data-[side=bottom]:data-[align=center]:left-1/2 data-[side=bottom]:data-[align=center]:-translate-x-1/2 data-[side=bottom]:data-[align=start]:left-(--arrow-offset) data-[side=bottom]:data-[align=start]:-translate-x-1/2 data-[side=bottom]:data-[align=end]:right-(--arrow-offset) data-[side=bottom]:data-[align=end]:translate-x-1/2",
      )}
      style={{
        "--arrow-offset": `${triggerWidth() / 2}px`,
      }}
    />
  );
};
