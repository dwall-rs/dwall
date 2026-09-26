/* ===== src/layout/ThemeToggleButton.tsx ===== */
// 职责：顶栏一键切换外观（亮 ⇄ 暗）；以当前解析结果取反，避免三态循环的空点击。
import { themeStore, toggle } from "@/store/theme.store";
import { t } from "@/i18n";
import { Button } from "~/components/ui/button";
import { Moon, Sun } from "lucide-solid";
import { Show } from "solid-js";

export function ThemeToggleButton() {
  return (
    <Button
      onClick={toggle}
      title={`${t("app.appearance.title")}: ${t(
        themeStore.resolved === "dark"
          ? "app.appearance.dark"
          : "app.appearance.light",
      )}`}
    >
      <Show
        when={themeStore.resolved === "dark"}
        fallback={<Sun class="size-4" />}
      >
        <Moon class="size-4" />
      </Show>
    </Button>
  );
}
