// Responsibility: keeps unsaved settings visible from anywhere on the settings page and
// gives the page the standard Ctrl+S shortcut. Rendered by the shell so it does not
// scroll away with the section that was edited.
import { createEffect, createMemo, onCleanup, Show } from "solid-js";
import { Button } from "@/components/ui/button";
import { t } from "@/i18n";
import {
  discard,
  isConfigDirty,
  save,
  settingsStore,
} from "@/store/settings.store";
import { uiStore } from "@/store/ui.store";
import { LoaderCircle, Save, TriangleAlert } from "lucide-solid";

export function SettingsSaveBar() {
  const dirty = createMemo(() => isConfigDirty(settingsStore));
  const visible = createMemo(
    () =>
      uiStore.view === "settings" &&
      settingsStore.status === "ready" &&
      (dirty() || !!settingsStore.saveError),
  );

  const saveNow = () => {
    if (settingsStore.saving) return;

    // Number fields commit on blur/Enter. Ctrl+S can arrive while one is focused;
    // blur first so the shortcut saves what the user sees, not the previous value.
    if (document.activeElement instanceof HTMLElement) {
      document.activeElement.blur();
    }

    if (!dirty()) return;
    void save();
  };

  createEffect(() => {
    if (uiStore.view !== "settings") return;

    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
        event.preventDefault();
        saveNow();
      }
    };
    document.addEventListener("keydown", onKeyDown);
    onCleanup(() => document.removeEventListener("keydown", onKeyDown));
  });

  return (
    <Show when={visible()}>
      <div class="pointer-events-none fixed inset-x-0 bottom-0 z-40 flex justify-center px-6 pb-5">
        <div class="pointer-events-auto flex w-full max-w-[560px] items-center gap-3 rounded-[16px] border border-border bg-popover/95 px-4 py-3 shadow-[0_18px_50px_rgba(0,0,0,.22)] backdrop-blur-md">
          <span class="min-w-0 flex-1">
            <Show
              when={!settingsStore.saveError}
              fallback={
                <span class="flex items-center gap-1.5 text-[12px] text-destructive">
                  <TriangleAlert class="size-3.5 shrink-0" />
                  {t("settings.saveFailed")}
                </span>
              }
            >
              <span class="flex items-center gap-1.5 text-[12px] text-muted-foreground">
                <span class="size-1.5 rounded-full bg-warning animate-bdot" />
                {t("settings.unsaved")}
              </span>
            </Show>
          </span>
          <Button
            variant="ghost"
            size="sm"
            disabled={
              (!dirty() && !settingsStore.saveError) || settingsStore.saving
            }
            onClick={discard}
          >
            {t("settings.discard")}
          </Button>
          <Button
            size="sm"
            disabled={!dirty() || settingsStore.saving}
            onClick={saveNow}
          >
            <Show
              when={settingsStore.saving}
              fallback={<Save class="size-3.5" />}
            >
              <LoaderCircle class="size-3.5 animate-spin" />
            </Show>
            {settingsStore.saving ? t("settings.saving") : t("settings.save")}
          </Button>
        </div>
      </div>
    </Show>
  );
}
