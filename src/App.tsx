// Responsibility: root component, renders the shell only; holds no business state (state lives in the individual stores).
import { AppShell } from "@/layout/AppShell";
import { ErrorBoundary, onCleanup, onMount } from "solid-js";
import { Button } from "@/components/ui/button";
import { applySystemLocale, t } from "@/i18n";
import { bootstrap } from "./store/bootstrap";
import { refresh as refreshEngine } from "./store/engine.store";
import { checkOnStartup as checkUpdatesOnStartup } from "./store/update.store";
import { getSystemLocale, showWindow } from "./ipc";

export default function App() {
  onMount(async () => {
    // WebView2 can report an English navigator on a Chinese Windows install. Ask
    // Windows for the user locale before showing the hidden production window, so
    // the correct language is in the very first paint; a browser-detected locale is
    // already in place if this fails.
    try {
      const systemLocale = await getSystemLocale();
      if (systemLocale) applySystemLocale(systemLocale);
    } catch {
      // The synchronous browser locale is a safe fallback.
    }

    if (import.meta.env.PROD) showWindow("main");

    refreshEngine();
    // The engine may start after the window is created, or be started/stopped externally; periodically align with the real state.
    const timer = setInterval(refreshEngine, 5000);
    onCleanup(() => clearInterval(timer));

    // Loads both stores and derives the editing state. The same step runs again from
    // either error panel's retry, so a failed load cannot leave the pool empty.
    await bootstrap();

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
