// Responsibility: Settings view — title + intro + the sections (each section keeps a single responsibility).
import { AppearanceSection } from "@/components/settings/AppearanceSection";
import { EngineSection } from "@/components/settings/EngineSection";
import { PathsSection } from "@/components/settings/PathsSection";
import { AboutSection } from "@/components/settings/AboutSection";
import { t } from "@/i18n";
import { createMemo, Show } from "solid-js";
import {
  discard,
  isConfigDirty,
  load,
  save,
  settingsStore,
} from "~/store/settings.store";
import { clsx } from "~/utils";
import { Button } from "~/components/ui/button";
import { Loader2, Save, TriangleAlert } from "lucide-solid";

export function SettingsView() {
  const dirty = createMemo(() => isConfigDirty(settingsStore));
  return (
    <div class="mx-auto max-w-190 px-7 pb-16 pt-8">
      <div class="mb-1.5 flex items-center gap-3">
        <h2 class="font-display text-[26px] font-extrabold tracking-tight">
          {t("settings.title")}
        </h2>
        <span class="font-mono text-[12px] text-muted-foreground">
          {t("settings.subtitle")}
        </span>

        <Show when={settingsStore.status === "ready"}>
          <div class="ml-auto flex items-center gap-2">
            <Show when={settingsStore.saveError}>
              <span class="text-[12px] text-destructive">
                {t("settings.saveFailed")}
              </span>
            </Show>

            <span
              class={clsx(
                "flex items-center gap-1.5 text-[12px] text-warning transition-opacity",
                dirty() ? "opacity-100" : "opacity-0",
              )}
            >
              <span class="size-1.5 rounded-full bg-warning animate-bdot" />
              {t("settings.unsaved")}
            </span>
            <Button
              variant="ghost"
              size="sm"
              disabled={!dirty}
              onClick={discard}
            >
              {t("settings.discard")}
            </Button>
            <Button
              size="sm"
              disabled={!dirty() || settingsStore.saving}
              onClick={save}
              class={clsx(
                dirty() && !settingsStore.saving && "animate-breathe",
              )}
            >
              <Show
                when={settingsStore.saving}
                fallback={
                  <>
                    <Save class="size-3.5" />
                    {t("settings.save")}
                  </>
                }
              >
                <Loader2 class="size-3.5 animate-spin" />
                {t("settings.saving")}
              </Show>
            </Button>
          </div>
        </Show>
      </div>
      <p class="mb-7 max-w-145 text-[13px] leading-relaxed text-muted-foreground">
        {t("settings.intro")}
      </p>

      {/*
        A configuration that could not be read used to render as an ordinary page
        with every switch off and an empty themes directory, which reads as "my
        settings are gone" rather than "the file could not be read" — and left the
        sections operable with no configuration behind them.
      */}
      <Show when={settingsStore.status === "error"}>
        <div class="mb-7 flex items-start gap-3 rounded-[15px] border border-destructive/40 bg-card p-5">
          <TriangleAlert class="mt-0.5 size-4 shrink-0 text-destructive" />
          <div class="min-w-0 flex-1">
            <p class="text-[13px] font-medium">{t("settings.loadFailed")}</p>
            <p
              data-selectable
              class="mt-1 text-[12px] leading-relaxed break-all text-muted-foreground"
            >
              {settingsStore.error}
            </p>
            <Button
              variant="outline"
              size="sm"
              class="mt-3"
              onClick={() => void load()}
            >
              {t("settings.retry")}
            </Button>
          </div>
        </div>
      </Show>

      <Show
        when={
          settingsStore.status === "idle" || settingsStore.status === "loading"
        }
      >
        <p class="mb-7 text-[13px] text-muted-foreground">
          {t("common.loading")}
        </p>
      </Show>

      <Show when={settingsStore.status === "ready"}>
        <AppearanceSection />
        <EngineSection />
        <PathsSection />
      </Show>

      {/* Independent of the configuration: the log directory and the update check
          are exactly what a broken configuration needs to be diagnosed. */}
      <AboutSection />
    </div>
  );
}
