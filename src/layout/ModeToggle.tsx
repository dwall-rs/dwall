// Responsibility: fixed/random mode segmented control. Changing it only changes the
// editing context; the applied mode is written when the user applies a theme or saves the pool.
import { For } from "solid-js";
import { modeStore, setMode } from "@/store/mode.store";
import type { Mode } from "@/models/types";
import { t } from "@/i18n";
import { clsx } from "@/utils";
import { CalendarDays, Monitor } from "lucide-solid";

const ITEMS: {
  value: Mode;
  key: "app.mode.fixed" | "app.mode.random";
  hint: "app.mode.fixedHint" | "app.mode.randomHint";
  Icon: typeof Monitor;
}[] = [
  {
    value: "fixed",
    key: "app.mode.fixed",
    hint: "app.mode.fixedHint",
    Icon: Monitor,
  },
  {
    value: "random",
    key: "app.mode.random",
    hint: "app.mode.randomHint",
    Icon: CalendarDays,
  },
];

export function ModeToggle() {
  return (
    <div
      role="group"
      aria-label={t("workspace.editing")}
      class="inline-flex items-center gap-0.5 rounded-full border border-border bg-muted/65 p-1"
    >
      <For each={ITEMS}>
        {(item) => {
          const active = () => modeStore.mode === item.value;
          return (
            <button
              type="button"
              aria-pressed={active()}
              title={t(item.hint)}
              onClick={() => setMode(item.value)}
              class={clsx(
                "flex h-7 cursor-pointer items-center gap-1.5 rounded-full px-3 text-[12.5px] font-medium transition-colors outline-none focus-visible:ring-3 focus-visible:ring-ring/50",
                active()
                  ? "bg-card text-foreground shadow-[0_1px_2px_rgba(0,0,0,.08)]"
                  : "text-muted-foreground hover:text-foreground",
              )}
            >
              <item.Icon class="size-3.5" />
              {t(item.key)}
            </button>
          );
        }}
      </For>
    </div>
  );
}
