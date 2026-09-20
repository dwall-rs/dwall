/* ===== src/components/layout/ThemeCycleButton.tsx ===== */
// 职责：顶栏一键循环外观（亮→暗→系统）；图标随解析结果，title 显示用户意图。
import { themeStore, cycle } from "@/store/theme.store";
import { useResolvedTheme } from "@/hooks/useResolvedTheme";
import { Button } from "~/components/ui/button";
import { Moon, Sun } from "lucide-solid";

const LABEL = { light: "亮色", dark: "暗色", system: "跟随系统" } as const;

export function ThemeCycleButton() {
  const resolved = useResolvedTheme();
  const Icon = resolved === "dark" ? Moon : Sun;

  return (
    <Button
      onClick={cycle}
      title={`外观：${LABEL[themeStore.mode]}（当前 ${resolved === "dark" ? "暗" : "亮"}）`}
    >
      <Icon class="size-4" />
    </Button>
  );
}
