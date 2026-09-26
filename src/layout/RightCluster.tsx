// 职责：顶栏右侧按钮排布；新增「主题库抽屉」开关，仅在最小档（<xl）可见。
import { uiStore, toggleLib } from "@/store/ui.store";
import { t } from "@/i18n";
import { EngineButton } from "./EngineButton";
import { ThemeToggleButton } from "./ThemeToggleButton";
import { SettingsButton } from "./SettingsButton";
import { PanelRight } from "lucide-solid";
import { Button } from "@/components/ui/button";

export function RightCluster() {
  return (
    <div class="ml-auto flex items-center gap-2.5">
      {/* 最小档：主题库收为抽屉，用此按钮展开/收起；xl+ 隐藏（库已内联） */}
      <Button
        onClick={toggleLib}
        title={
          uiStore.libOpen ? t("library.toggleOpen") : t("library.toggleClosed")
        }
        class="xl:hidden"
      >
        <PanelRight class="size-4" />
      </Button>
      <EngineButton />
      <span class="h-5.5 w-px bg-border-2" />
      <ThemeToggleButton />
      <SettingsButton />
    </div>
  );
}
