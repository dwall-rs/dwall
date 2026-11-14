// Responsibility: display scope — one shared theme or a theme per screen, with each
// display's applied state and a direct route to pick its wallpaper.
import { For, Show, createMemo } from "solid-js";
import { PageHeader } from "@/layout/PageHeader";
import { ModeToggle } from "@/layout/ModeToggle";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { t } from "@/i18n";
import { catalogStore } from "@/store/catalog.store";
import { monitorList, themeById } from "@/store/catalog.selectors";
import {
  fixedStore,
  isApplied,
  isConfiguredTheme,
  selectMon,
  themeForMon,
  toggleApply,
  toggleUnified,
} from "@/store/fixed.store";
import { modeStore } from "@/store/mode.store";
import { setView } from "@/store/ui.store";
import { ThemeThumbnail } from "@/scene/ThemeThumbnail";
import { clsx } from "@/utils";
import { ArrowRight, Monitor, MonitorOff, Square } from "lucide-solid";

export function DisplaysView() {
  const monitors = createMemo(() =>
    monitorList().filter((monitor) => monitor.id !== "all"),
  );
  const fixedMode = () => modeStore.mode === "fixed";
  const unifiedTheme = createMemo(() => themeById(themeForMon("all")));

  const choose = (id: string) => {
    if (fixedStore.allUnified) toggleUnified();
    selectMon(id);
    setView("themes");
  };

  return (
    <div class="mx-auto w-full max-w-[1720px] px-7 pb-10 pt-6 animate-fade-up">
      <PageHeader title={t("nav.displays")} subtitle={t("displays.subtitle")}>
        <ModeToggle />
        <Button variant="outline" size="sm" onClick={() => setView("themes")}>
          {t("nav.themes")}
        </Button>
      </PageHeader>

      <Show when={modeStore.mode === "random"}>
        <div class="mb-5 rounded-[12px] border border-border bg-card/60 px-3.5 py-2.5 text-[11.5px] leading-relaxed text-muted-foreground">
          {t("displays.randomNote")}
        </div>
      </Show>

      <section class="mb-6 overflow-hidden rounded-[16px] border border-border bg-card">
        <div class="flex flex-wrap items-center gap-4 px-4 py-4">
          <div class="grid size-9 shrink-0 place-items-center rounded-[10px] bg-accent text-primary">
            <Monitor class="size-4.5" />
          </div>
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2">
              <h2 class="text-[13.5px] font-semibold">
                {t("displays.unified")}
              </h2>
              <Show when={isApplied("all")}>
                <span class="rounded-full bg-success-dim px-2 py-0.5 font-mono text-[9px] text-success">
                  {t("displays.applied")}
                </span>
              </Show>
            </div>
            <p class="mt-1 max-w-[72ch] text-[11.5px] leading-relaxed text-muted-foreground">
              {t("displays.unifiedDesc")}
            </p>
          </div>
          <Switch
            aria-label={t("displays.unified")}
            checked={fixedStore.allUnified}
            disabled={!fixedMode() || fixedStore.applying}
            onCheckedChange={() => toggleUnified()}
          />
        </div>

        <div class="flex flex-wrap items-center gap-3 border-t border-border bg-muted/30 px-4 py-3">
          <div class="h-9 w-16 shrink-0 overflow-hidden rounded-md border border-border">
            <ThemeThumbnail
              themeId={themeForMon("all") ?? ""}
              fallback={
                <div class="grid h-full w-full place-items-center bg-card">
                  <Monitor class="size-4 text-muted-foreground" />
                </div>
              }
            />
          </div>
          <div class="min-w-0 flex-1">
            <div class="text-[12px] font-medium">{unifiedTheme().name}</div>
            <div class="mt-0.5 font-mono text-[9.5px] text-muted-foreground">
              {isApplied("all")
                ? t("displays.applied")
                : isConfiguredTheme("all")
                  ? t("workspace.readyOn", { name: t("workspace.allDisplays") })
                  : t("displays.notApplied")}
            </div>
          </div>
          <Button
            variant="ghost"
            size="sm"
            disabled={!fixedMode() || fixedStore.applying}
            onClick={() => {
              if (!fixedStore.allUnified) toggleUnified();
              setView("themes");
            }}
          >
            {t("displays.chooseTheme")}
            <ArrowRight class="size-3.5" />
          </Button>
          <Button
            size="sm"
            variant={isApplied("all") ? "destructive" : "primary"}
            disabled={
              !fixedMode() ||
              !fixedStore.allUnified ||
              fixedStore.applying ||
              (!isApplied("all") && !!unifiedTheme()?.needsDownload)
            }
            onClick={() => void toggleApply("all")}
          >
            <Show
              when={isApplied("all")}
              fallback={<ArrowRight class="size-3.5" />}
            >
              <Square class="size-3 fill-current" />
            </Show>
            {isApplied("all") ? t("displays.stopAll") : t("displays.applyAll")}
          </Button>
        </div>
      </section>

      <div class="mb-3 flex items-baseline justify-between">
        <h2 class="text-[12px] font-semibold">{t("displays.individual")}</h2>
        <span class="font-mono text-[10px] text-muted-foreground">
          {t("displays.monitorCount", { n: monitors().length })}
        </span>
      </div>

      <Show
        when={monitors().length > 0}
        fallback={
          <div class="flex flex-col items-center justify-center gap-2 rounded-[16px] border border-dashed border-border-2 py-16 text-center">
            <MonitorOff class="size-8 text-muted-foreground/40" />
            <p class="font-display text-[15px] font-semibold">
              {t("displays.noMonitors")}
            </p>
            <p class="max-w-[38ch] text-[11.5px] text-muted-foreground">
              {t("displays.noMonitorsDesc")}
            </p>
          </div>
        }
      >
        <div class="grid gap-3 min-[1100px]:grid-cols-2">
          <For each={monitors()}>
            {(monitor) => {
              const displayThemeId = () =>
                themeForMon(fixedStore.allUnified ? "all" : monitor.id);
              const theme = createMemo(() => themeById(displayThemeId()));
              const applied = createMemo(() => isApplied(monitor.id));
              const configured = createMemo(() =>
                isConfiguredTheme(monitor.id),
              );
              return (
                <article class="overflow-hidden rounded-[16px] border border-border bg-card">
                  <div
                    class={clsx(
                      "relative aspect-[16/7] overflow-hidden bg-muted",
                      fixedStore.allUnified && "opacity-55",
                    )}
                  >
                    <ThemeThumbnail
                      themeId={displayThemeId() ?? ""}
                      fallback={
                        <div class="grid h-full w-full place-items-center bg-card">
                          <Monitor class="size-6 text-muted-foreground" />
                        </div>
                      }
                    />
                    <div class="pointer-events-none absolute inset-x-0 bottom-0 bg-linear-to-t from-black/75 to-transparent px-3 pb-2 pt-8">
                      <div class="font-display text-[14px] font-semibold text-white">
                        {monitor.name}
                      </div>
                    </div>
                  </div>

                  <div class="flex flex-wrap items-center gap-3 px-3.5 py-3">
                    <div class="min-w-0 flex-1">
                      <div class="font-mono text-[9.5px] text-muted-foreground">
                        {t("displays.currentTheme")}
                      </div>
                      <div class="mt-0.5 truncate text-[12.5px] font-medium">
                        {theme().name}
                      </div>
                      <div
                        class={clsx(
                          "mt-1 font-mono text-[9.5px]",
                          applied() ? "text-success" : "text-muted-foreground",
                        )}
                      >
                        {fixedStore.allUnified
                          ? t("displays.sharedTheme")
                          : applied()
                            ? t("displays.applied")
                            : configured()
                              ? t("workspace.readyOn", { name: monitor.name })
                              : t("displays.notApplied")}
                      </div>
                    </div>

                    <Button
                      variant="ghost"
                      size="sm"
                      disabled={
                        !fixedMode() ||
                        fixedStore.allUnified ||
                        fixedStore.applying
                      }
                      onClick={() => choose(monitor.id)}
                    >
                      {t("displays.chooseTheme")}
                    </Button>

                    <Show when={!fixedStore.allUnified}>
                      <Switch
                        aria-label={monitor.name}
                        checked={applied()}
                        disabled={
                          !fixedMode() ||
                          fixedStore.applying ||
                          (!applied() && !!theme()?.needsDownload)
                        }
                        onCheckedChange={() => void toggleApply(monitor.id)}
                      />
                    </Show>
                  </div>
                </article>
              );
            }}
          </For>
        </div>
      </Show>

      <Show when={catalogStore.status === "loading"}>
        <p class="mt-6 text-center text-[12px] text-muted-foreground">
          {t("common.loading")}
        </p>
      </Show>
    </div>
  );
}
