// Responsibility: Button layout on the right of the top bar; adds the "theme library drawer" toggle, visible only at the smallest tier (<xl).
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
      {/* Smallest tier: the theme library collapses into a drawer, expanded/collapsed by this button; hidden at xl+ (the library is inline) */}
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
