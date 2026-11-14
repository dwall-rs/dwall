// Responsibility: persistent navigation and live engine status. The window has one
// rail, so a page is never hidden behind a drawer and the engine is always in reach.
import { For, Show } from "solid-js";
import { t } from "@/i18n";
import { uiStore, setView } from "@/store/ui.store";
import { catalogStore } from "@/store/catalog.store";
import { availableThemes } from "@/store/catalog.selectors";
import { isConfigDirty, settingsStore } from "@/store/settings.store";
import { setMode, themeStore } from "@/store/theme.store";
import type { ThemeMode, View } from "@/models/types";
import { clsx } from "@/utils";
import {
  Images,
  Monitor,
  Moon,
  Settings,
  Sun,
  SunMedium,
  SunMoon,
} from "lucide-solid";
import Logo from "@/assets/dwall.svg";
import { EnginePanel } from "./EnginePanel";

interface NavItem {
  view: View;
  label: "nav.wallpaper" | "nav.themes" | "nav.displays" | "nav.settings";
  Icon: typeof SunMedium;
  badge?: () => string | number | undefined;
  attention?: () => boolean;
}

const ITEMS: NavItem[] = [
  { view: "wallpaper", label: "nav.wallpaper", Icon: SunMedium },
  {
    view: "themes",
    label: "nav.themes",
    Icon: Images,
    badge: () =>
      availableThemes().length > 0 ? availableThemes().length : undefined,
  },
  {
    view: "displays",
    label: "nav.displays",
    Icon: Monitor,
    badge: () =>
      catalogStore.monitors.length > 0
        ? catalogStore.monitors.length
        : undefined,
  },
  {
    view: "settings",
    label: "nav.settings",
    Icon: Settings,
    attention: () => isConfigDirty(settingsStore),
  },
];

const APPEARANCE_MODES: {
  value: ThemeMode;
  key: "app.appearance.light" | "app.appearance.dark" | "app.appearance.system";
  Icon: typeof Sun;
}[] = [
  { value: "light", key: "app.appearance.light", Icon: Sun },
  { value: "dark", key: "app.appearance.dark", Icon: Moon },
  { value: "system", key: "app.appearance.system", Icon: Monitor },
];

export function Sidebar() {
  const appearanceLabel = () => {
    const current =
      APPEARANCE_MODES.find((item) => item.value === themeStore.mode) ??
      APPEARANCE_MODES[2];
    return t(current.key);
  };

  return (
    <aside class="flex w-[236px] shrink-0 flex-col border-r border-sidebar-border bg-sidebar text-sidebar-foreground">
      <div class="flex items-center gap-3 px-5 pb-4 pt-5">
        <img
          src={Logo}
          alt={t("settings.about.logoAlt")}
          class="size-9 shrink-0 drag-none"
        />
        <div class="min-w-0">
          <div class="font-display text-[17px] font-semibold leading-none tracking-[-0.02em]">
            DWALL
          </div>
          <div class="mt-1.5 truncate text-[10.5px] leading-none text-muted-foreground">
            {t("app.tagline")}
          </div>
        </div>
      </div>

      <nav class="flex flex-col gap-0.5 px-3 pt-1">
        <For each={ITEMS}>
          {(item) => {
            const active = () => uiStore.view === item.view;
            return (
              <button
                type="button"
                aria-current={active() ? "page" : undefined}
                onClick={() => setView(item.view)}
                class={clsx(
                  "group relative flex h-10 cursor-pointer items-center gap-2.5 rounded-[10px] px-2.5 text-left text-[13px] font-medium transition-colors outline-none focus-visible:ring-3 focus-visible:ring-ring/50",
                  active()
                    ? "bg-sidebar-accent text-sidebar-foreground"
                    : "text-muted-foreground hover:bg-sidebar-accent/60 hover:text-sidebar-foreground",
                )}
              >
                <span
                  class={clsx(
                    "absolute inset-y-2 left-0 w-[2px] rounded-full bg-solar transition-opacity",
                    active() ? "opacity-100" : "opacity-0",
                  )}
                />
                <item.Icon
                  class={clsx(
                    "size-4 shrink-0 transition-colors",
                    active() ? "text-solar" : "text-muted-foreground",
                  )}
                />
                <span class="min-w-0 flex-1 truncate">{t(item.label)}</span>
                <Show when={item.badge?.()}>
                  {(badge) => (
                    <span class="rounded-full bg-sidebar px-1.5 py-0.5 font-mono text-[10px] leading-none text-muted-foreground">
                      {badge()}
                    </span>
                  )}
                </Show>
                <Show when={item.attention?.()}>
                  <span class="size-1.5 rounded-full bg-warning" />
                </Show>
              </button>
            );
          }}
        </For>
      </nav>

      <div class="mt-auto flex flex-col gap-2 pb-3">
        <EnginePanel />

        <div class="px-3 pt-1">
          <div class="flex h-10 items-center gap-2 rounded-[10px] px-2.5">
            <SunMoon class="size-4 shrink-0 text-muted-foreground" />
            <div class="min-w-0 flex-1">
              <div class="text-[12.5px] leading-tight text-muted-foreground">
                {t("app.appearance.title")}
              </div>
              <div class="mt-0.5 truncate font-mono text-[9.5px] leading-tight text-muted-foreground/80">
                {appearanceLabel()}
              </div>
            </div>
            <div
              role="group"
              aria-label={t("app.appearance.title")}
              class="flex shrink-0 items-center gap-0.5 rounded-full border border-sidebar-border bg-sidebar p-0.5"
            >
              <For each={APPEARANCE_MODES}>
                {(item) => {
                  const active = () => themeStore.mode === item.value;
                  return (
                    <button
                      type="button"
                      aria-label={t(item.key)}
                      aria-pressed={active()}
                      title={t(item.key)}
                      onClick={() => setMode(item.value)}
                      class={clsx(
                        "grid size-6 cursor-pointer place-items-center rounded-full transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring/50",
                        active()
                          ? "bg-card text-foreground shadow-[0_1px_2px_rgba(0,0,0,.1)]"
                          : "text-muted-foreground hover:text-foreground",
                      )}
                    >
                      <item.Icon class="size-3.5" />
                    </button>
                  );
                }}
              </For>
            </div>
          </div>
        </div>
      </div>
    </aside>
  );
}
