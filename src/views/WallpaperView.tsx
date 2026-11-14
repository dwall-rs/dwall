// Responsibility: the wallpaper surface — context (mode + scope) and the stage that
// shows the current frame.
import { For, Show } from "solid-js";
import { PageHeader } from "@/layout/PageHeader";
import { ModeToggle } from "@/layout/ModeToggle";
import { Button } from "@/components/ui/button";
import { t } from "@/i18n";
import { modeStore } from "@/store/mode.store";
import { fixedStore, selectMon, toggleUnified } from "@/store/fixed.store";
import { monitorList } from "@/store/catalog.selectors";
import { setView } from "@/store/ui.store";
import { clsx } from "@/utils";
import { FixedStage } from "@/stage/FixedStage";
import { RandomStage } from "@/stage/RandomStage";
import { Images, Monitor } from "lucide-solid";

export function WallpaperView() {
  const fixed = () => modeStore.mode === "fixed";

  return (
    <div class="mx-auto w-full max-w-[1720px] px-7 pb-10 pt-6 animate-fade-up">
      <PageHeader
        title={t("nav.wallpaper")}
        subtitle={
          fixed() ? t("workspace.subtitle") : t("workspace.randomSubtitle")
        }
      >
        <ModeToggle />
        <Button variant="outline" size="sm" onClick={() => setView("themes")}>
          <Images class="size-3.5" />
          {fixed() ? t("nav.themes") : t("workspace.editPool")}
        </Button>
      </PageHeader>

      <Show when={fixed()}>
        <div class="mb-5 flex flex-wrap items-center gap-2">
          <span class="mr-1 text-[11px] font-medium text-muted-foreground">
            {t("workspace.editing")}
          </span>

          <button
            type="button"
            aria-pressed={fixedStore.allUnified}
            onClick={() => {
              if (!fixedStore.allUnified) toggleUnified();
            }}
            class={clsx(
              "flex h-7 cursor-pointer items-center gap-1.5 rounded-full border px-3 text-[11.5px] font-medium transition-colors outline-none focus-visible:ring-3 focus-visible:ring-ring/50",
              fixedStore.allUnified
                ? "border-primary/35 bg-primary/10 text-foreground"
                : "border-border text-muted-foreground hover:border-border-2 hover:text-foreground",
            )}
          >
            <Monitor class="size-3.5" />
            {t("workspace.allDisplays")}
          </button>

          <For each={monitorList().filter((monitor) => monitor.id !== "all")}>
            {(monitor) => {
              const active = () =>
                !fixedStore.allUnified && fixedStore.curMon === monitor.id;
              return (
                <button
                  type="button"
                  aria-pressed={active()}
                  onClick={() => {
                    if (fixedStore.allUnified) toggleUnified();
                    selectMon(monitor.id);
                  }}
                  class={clsx(
                    "flex h-7 max-w-[220px] cursor-pointer items-center gap-1.5 truncate rounded-full border px-3 text-[11.5px] font-medium transition-colors outline-none focus-visible:ring-3 focus-visible:ring-ring/50",
                    active()
                      ? "border-primary/35 bg-primary/10 text-foreground"
                      : "border-border text-muted-foreground hover:border-border-2 hover:text-foreground",
                  )}
                >
                  <span class="truncate">{monitor.name}</span>
                </button>
              );
            }}
          </For>

          <button
            type="button"
            onClick={() => setView("displays")}
            class="ml-auto cursor-pointer text-[11px] font-medium text-muted-foreground hover:text-foreground hover:underline"
          >
            {t("workspace.manageDisplays")}
          </button>
        </div>
      </Show>

      <Show when={fixed()} fallback={<RandomStage />}>
        <FixedStage />
      </Show>
    </div>
  );
}
