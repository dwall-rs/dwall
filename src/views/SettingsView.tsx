// Responsibility: settings surface — a section rail and one content column. Every
// section stays mounted so in-page links and the update notification can reach it.
import { For, Show, createEffect, createSignal, onCleanup } from "solid-js";
import { PageHeader } from "@/layout/PageHeader";
import { Button } from "@/components/ui/button";
import { AppearanceSection } from "@/components/settings/AppearanceSection";
import { EngineSection } from "@/components/settings/EngineSection";
import { PathsSection } from "@/components/settings/PathsSection";
import { AboutSection } from "@/components/settings/AboutSection";
import { t } from "@/i18n";
import { isConfigDirty, load, settingsStore } from "@/store/settings.store";
import { clsx } from "@/utils";
import {
  Check,
  FileText,
  Folder,
  LoaderCircle,
  Sparkles,
  Sun,
  TriangleAlert,
} from "lucide-solid";

const SECTIONS = [
  {
    id: "settings-appearance",
    key: "settings.appearance.title" as const,
    Icon: Sparkles,
  },
  { id: "settings-engine", key: "settings.engine.title" as const, Icon: Sun },
  { id: "settings-paths", key: "settings.paths.title" as const, Icon: Folder },
  {
    id: "settings-about",
    key: "settings.about.title" as const,
    Icon: FileText,
  },
];

export function SettingsView() {
  const [active, setActive] = createSignal(SECTIONS[0].id);
  const dirty = () => isConfigDirty(settingsStore);
  const ready = () => settingsStore.status === "ready";
  const failed = () => settingsStore.status === "error";

  const jump = (id: string) => {
    setActive(id);
    document.getElementById(id)?.scrollIntoView({
      behavior: "smooth",
      block: "start",
    });
  };

  // Re-runs when the configuration becomes ready: the sections do not exist while it
  // is loading. A scroll-position rule rather than an IntersectionObserver, because the
  // last section can never reach a narrow observation band at the bottom of the page.
  createEffect(() => {
    if (settingsStore.status !== "ready") return;

    const root = document.querySelector<HTMLElement>("[data-scroll-root]");
    if (!root) return;

    const update = () => {
      const threshold = root.scrollTop + root.clientHeight * 0.22;
      let current = SECTIONS[0].id;

      for (const section of SECTIONS) {
        const element = document.getElementById(section.id);
        if (!element) continue;
        const top =
          element.getBoundingClientRect().top -
          root.getBoundingClientRect().top +
          root.scrollTop;
        if (top <= threshold) current = section.id;
      }

      if (root.scrollTop + root.clientHeight >= root.scrollHeight - 8) {
        current = SECTIONS[SECTIONS.length - 1].id;
      }
      setActive(current);
    };

    update();
    root.addEventListener("scroll", update, { passive: true });
    onCleanup(() => root.removeEventListener("scroll", update));
  });

  return (
    <div class="mx-auto w-full max-w-[1480px] px-7 pb-28 pt-6 animate-fade-up">
      <PageHeader title={t("settings.title")} subtitle={t("settings.subtitle")}>
        <span
          class={clsx(
            "flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-[10.5px] font-medium",
            failed()
              ? "border-destructive/35 bg-destructive-dim text-destructive"
              : dirty()
                ? "border-warning/35 bg-warning-dim text-warning"
                : "border-border bg-card text-muted-foreground",
          )}
        >
          <Show
            when={ready() && !dirty()}
            fallback={
              failed() ? (
                <TriangleAlert class="size-3" />
              ) : !ready() ? (
                <LoaderCircle class="size-3 animate-spin" />
              ) : (
                <span class="size-1.5 rounded-full bg-warning animate-bdot" />
              )
            }
          >
            <Check class="size-3 text-success" />
          </Show>
          {failed()
            ? t("settings.loadFailed")
            : !ready()
              ? t("common.loading")
              : dirty()
                ? t("settings.unsaved")
                : t("settings.saved")}
        </span>
      </PageHeader>

      <div class="grid items-start gap-8 min-[1050px]:grid-cols-[196px_minmax(0,1fr)]">
        <nav
          aria-label={t("settings.navLabel")}
          class="flex gap-1 overflow-x-auto pb-1 min-[1050px]:sticky min-[1050px]:top-6 min-[1050px]:flex-col min-[1050px]:overflow-visible"
        >
          <For each={SECTIONS}>
            {(section) => {
              const isActive = () => active() === section.id;
              return (
                <button
                  type="button"
                  aria-current={isActive() ? "true" : undefined}
                  onClick={() => jump(section.id)}
                  class={clsx(
                    "group flex h-9 shrink-0 cursor-pointer items-center gap-2 rounded-[9px] px-2.5 text-left text-[12px] font-medium transition-colors outline-none focus-visible:ring-3 focus-visible:ring-ring/50",
                    isActive()
                      ? "bg-card text-foreground shadow-[inset_0_0_0_1px_var(--border)]"
                      : "text-muted-foreground hover:bg-card/60 hover:text-foreground",
                  )}
                >
                  <section.Icon
                    class={clsx(
                      "size-3.5 shrink-0",
                      isActive() ? "text-solar" : "text-muted-foreground",
                    )}
                  />
                  <span class="truncate">{t(section.key)}</span>
                </button>
              );
            }}
          </For>
        </nav>

        <div class="min-w-0 max-w-[820px]">
          <Show when={failed()}>
            <div class="mb-8 rounded-[16px] border border-destructive/40 bg-card p-4">
              <div class="flex items-start gap-2.5">
                <TriangleAlert class="mt-0.5 size-4 shrink-0 text-destructive" />
                <div class="min-w-0 flex-1">
                  <p class="text-[13px] font-semibold">
                    {t("settings.loadFailed")}
                  </p>
                  <p
                    data-selectable
                    class="mt-1 break-all font-mono text-[10.5px] leading-relaxed text-muted-foreground"
                  >
                    {settingsStore.error}
                  </p>
                  <Button
                    variant="outline"
                    size="sm"
                    class="mt-2.5"
                    onClick={() => void load()}
                  >
                    {t("settings.retry")}
                  </Button>
                </div>
              </div>
            </div>
          </Show>

          <Show when={!ready() && !failed()}>
            <p class="mb-8 text-[12px] text-muted-foreground">
              {t("common.loading")}
            </p>
          </Show>

          <Show when={ready()}>
            <AppearanceSection />
            <EngineSection />
            <PathsSection />
          </Show>
          <AboutSection />
        </div>
      </div>
    </div>
  );
}
