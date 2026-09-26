import { cva } from "class-variance-authority";

/**
 * ToggleGroupItem 位于 ToggleGroup 内时需要的“拼接态”样式。
 *
 * 依赖根元素上的钩子:
 * - `group-data-[spacing=0]/toggle-group`:spacing 为 0 时相邻 item 拼接为整体
 * - `data-[spacing=0]`:item 自身的 data-spacing(shadcn 同样在 item 上输出)
 * - `group-data-horizontal|vertical/toggle-group`:按方向决定首尾圆角/边框
 * - `data-[variant=outline]`:outline 拼接时只保留内侧边框
 *
 * variant / size 的基础样式仍由 `toggleVariants` 提供,这里只补“组合关系”。
 */
export const toggleGroupItemVariants = cva(
  "shrink-0 focus:z-10 focus-visible:z-10 group-data-[spacing=0]/toggle-group:rounded-none group-data-[spacing=0]/toggle-group:px-2 group-data-[spacing=0]/toggle-group:has-data-[icon=inline-end]:pr-1.5 group-data-[spacing=0]/toggle-group:has-data-[icon=inline-start]:pl-1.5 group-data-horizontal/toggle-group:data-[spacing=0]:first:rounded-l-lg group-data-vertical/toggle-group:data-[spacing=0]:first:rounded-t-lg group-data-horizontal/toggle-group:data-[spacing=0]:last:rounded-r-lg group-data-vertical/toggle-group:data-[spacing=0]:last:rounded-b-lg group-data-horizontal/toggle-group:data-[spacing=0]:data-[variant=outline]:border-l-0 group-data-vertical/toggle-group:data-[spacing=0]:data-[variant=outline]:border-t-0 group-data-horizontal/toggle-group:data-[spacing=0]:data-[variant=outline]:first:border-l group-data-vertical/toggle-group:data-[spacing=0]:data-[variant=outline]:first:border-t",
);
