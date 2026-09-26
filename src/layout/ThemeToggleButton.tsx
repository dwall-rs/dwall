/* ===== src/layout/ThemeCycleButton.tsx ===== */
// 职责：顶栏一键循环外观（亮→暗→系统）；图标随解析结果，title 显示用户意图。
import { themeStore, cycle } from "@/store/theme.store";
import { useResolvedTheme } from "@/hooks/useResolvedTheme";
import { t } from "@/i18n";
import { Button } from "~/components/ui/button";
import { Moon, Sun } from "lucide-solid";

const MODE_KEY = {
  light: "app.appearance.light",
  dark: "app.appearance.dark",
  system: "app.appearance.system",
} as const;

export function ThemeCycleButton() {
  const resolved = useResolvedTheme();
  const Icon = resolved === "dark" ? Moon : Sun;

  return (
    <Button
      onClick={cycle}
      title={`${t("app.appearance.title")}: ${t(MODE_KEY[themeStore.mode])}`}
    >
      <Icon class="size-4" />
    </Button>
  );
}
