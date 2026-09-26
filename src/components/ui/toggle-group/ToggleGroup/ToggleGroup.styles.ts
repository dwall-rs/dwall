import { cva } from "class-variance-authority";

/**
 * ToggleGroup 根容器样式。
 *
 * 根元素同时输出 `data-vertical` / `data-horizontal`(见 ToggleGroup.tsx),
 * 使 `data-vertical:*`、`group-data-vertical/toggle-group:*` 这类选择器生效 ——
 * item 的“拼接态”样式依赖后两者。
 */
export const toggleGroupVariants = cva(
  "group/toggle-group flex w-fit flex-row items-center gap-[--spacing(var(--gap))] rounded-lg data-[size=sm]:rounded-[min(var(--radius-md),10px)] data-vertical:flex-col data-vertical:items-stretch",
);
