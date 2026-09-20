// 职责：读取解析后的实际外观（供图标等需要「当前到底是亮是暗」的组件）。
import { themeStore } from "@/store/theme.store";
import { createMemo } from "solid-js";
export const useResolvedTheme = createMemo(() => themeStore.resolved);
