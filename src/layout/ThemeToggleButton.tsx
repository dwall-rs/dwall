/* ===== src/layout/ThemeToggleButton.tsx ===== */
// Responsibility: One-click appearance toggle in the top bar (light ⇄ dark); inverts the current resolved result to avoid dead clicks in the three-state cycle.
import { themeStore, toggle } from "@/store/theme.store";
import { t } from "@/i18n";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { Moon, Sun } from "lucide-solid";
import { Show } from "solid-js";

export function ThemeToggleButton() {
  const label = () =>
    `${t("app.appearance.title")}: ${t(
      themeStore.resolved === "dark"
        ? "app.appearance.dark"
        : "app.appearance.light",
    )}`;

  return (
    <Tooltip>
      <TooltipTrigger aria-label={label()} onClick={toggle}>
        <Show
          when={themeStore.resolved === "dark"}
          fallback={<Sun class="size-4" />}
        >
          <Moon class="size-4" />
        </Show>
      </TooltipTrigger>
      <TooltipContent>{label()}</TooltipContent>
    </Tooltip>
  );
}
