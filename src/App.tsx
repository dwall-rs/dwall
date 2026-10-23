// Responsibility: root component, renders the shell only; holds no business state (state lives in the individual stores).
import { AppShell } from "@/layout/AppShell";
import { ErrorBoundary, onCleanup, onMount } from "solid-js";
import { Button } from "@/components/ui/button";
import { t } from "@/i18n";
import { load as loadCatalog } from "./store/catalog.store";
import { availableThemeIds } from "./store/catalog.selectors";
import { refresh as refreshEngine } from "./store/engine.store";
import { syncFromConfig as syncFixed } from "./store/fixed.store";
import { syncFromConfig as syncRandom } from "./store/random.store";
import { load as loadSettings, settingsStore } from "./store/settings.store";
import { setMode } from "./store/mode.store";
import { checkOnStartup as checkUpdatesOnStartup } from "./store/update.store";
import { showWindow } from "./ipc";

export default function App() {
  onMount(async () => {
    if (import.meta.env.PROD) showWindow("main");

    refreshEngine();
    // The engine may start after the window is created, or be started/stopped externally; periodically align with the real state.
    const timer = setInterval(refreshEngine, 5000);
    onCleanup(() => clearInterval(timer));

    await Promise.all([loadSettings(), loadCatalog()]);

    const config = settingsStore.config;
    await syncFixed(config);
    // Only themes on disk can be applied, so only those can enter the pool.
    syncRandom(config, availableThemeIds());
    if (config?.wallpaper_mode?.mode === "random") setMode("random");

    // After the settings, because the check has to honour the configured proxy; not
    // awaited, so a slow network never delays the window.
    void checkUpdatesOnStartup();
  });
  return (
    <ErrorBoundary
      fallback={(error, reset) => (
        // Last-resort net: a render-time throw would otherwise leave a blank window.
        // The stage's resources guard their own error state; this catches the rest.
        <div class="flex h-screen flex-col items-center justify-center gap-4 p-8 text-center">
          <p class="font-display text-[15px] font-semibold">
            {t("common.operationFailed")}
          </p>
          <p
            data-selectable
            class="max-w-150 font-mono text-[11.5px] text-muted-foreground"
          >
            {String(error)}
          </p>
          <Button variant="outline" size="sm" onClick={reset}>
            {t("settings.retry")}
          </Button>
        </div>
      )}
    >
      <AppShell />
    </ErrorBoundary>
  );
}
