// 职责：根组件，仅渲染外壳；不持有任何业务状态（状态在各 store）。
import { AppShell } from "@/layout/AppShell";
import { onMount } from "solid-js";
import { catalogStore, load as loadCatalog } from "./store/catalog.store";
import { refresh as refreshEngine } from "./store/engine.store";
import { syncFromConfig as syncFixed } from "./store/fixed.store";
import { syncFromConfig as syncRandom } from "./store/random.store";
import { load as loadSettings, settingsStore } from "./store/settings.store";
import { setMode } from "./store/ui.store";

export default function App() {
  onMount(async () => {
    refreshEngine();
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
