// 职责：设置入口；在设置视图时高亮（active）。
import { uiStore, toggleView } from "@/store/ui.store";
import { t } from "@/i18n";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { Settings } from "lucide-solid";

export function SettingsButton() {
  return (
    <Tooltip>
      <TooltipTrigger
        aria-label={t("settings.title")}
        onClick={toggleView}
        variant={uiStore.view === "settings" ? "primary" : "outline"}
      >
        <Settings class="size-4" />
      </TooltipTrigger>
      <TooltipContent>{t("settings.title")}</TooltipContent>
    </Tooltip>
  );
}
