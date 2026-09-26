// 职责：顶栏右侧按钮排布；新增「主题库抽屉」开关，仅在最小档（<xl）可见。
import { uiStore, toggleLib } from "@/store/ui.store";
import { t } from "@/i18n";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { EngineButton } from "./EngineButton";
import { ThemeToggleButton } from "./ThemeToggleButton";
import { SettingsButton } from "./SettingsButton";
import { PanelRight } from "lucide-solid";

export function RightCluster() {
  const libLabel = () =>
    uiStore.libOpen ? t("library.toggleOpen") : t("library.toggleClosed");

  return (
    <div class="ml-auto flex items-center gap-2.5">
      {/* 最小档：主题库收为抽屉，用此按钮展开/收起；xl+ 隐藏（库已内联） */}
      <Tooltip>
        <TooltipTrigger
          aria-label={libLabel()}
          onClick={toggleLib}
          class="xl:hidden"
        >
          <PanelRight class="size-4" />
        </TooltipTrigger>
        <TooltipContent>{libLabel()}</TooltipContent>
      </Tooltip>
      <EngineButton />
      <span class="h-5.5 w-px bg-border-2" />
      <ThemeToggleButton />
      <SettingsButton />
    </div>
  );
}
