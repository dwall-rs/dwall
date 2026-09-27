// Responsibility: read the resolved appearance (for icons and other components that need to know whether it is currently light or dark).
import { themeStore } from "@/store/theme.store";
import { createMemo } from "solid-js";
export const useResolvedTheme = createMemo(() => themeStore.resolved);
