// Responsibility: root component, renders the shell only; holds no business state (state lives in the individual stores).
import { AppShell } from "@/layout/AppShell";
import { onCleanup, onMount } from "solid-js";
import { catalogStore, load as loadCatalog } from "./store/catalog.store";
import { refresh as refreshEngine } from "./store/engine.store";
import { syncFromConfig as syncFixed } from "./store/fixed.store";
import { syncFromConfig as syncRandom } from "./store/random.store";
import { load as loadSettings, settingsStore } from "./store/settings.store";
import { setMode } from "./store/mode.store";
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
    syncFixed(config);
    syncRandom(
      config,
      catalogStore.themes.map((t) => t.id),
    );
    if (config?.wallpaper_mode?.mode === "random") setMode("random");
  });
  return <AppShell />;
}
