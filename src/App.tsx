// 职责：根组件，仅渲染外壳；不持有任何业务状态（状态在各 store）。
import { AppShell } from "@/layout/AppShell";
import { onMount } from "solid-js";
import { catalogStore, load as loadCatalog } from "./store/catalog.store";
import { refresh as refreshEngine } from "./store/engine.store";
import { selectAll } from "./store/random.store";
import { load } from "./store/settings.store";

export default function App() {
  onMount(async () => {
    load();
    refreshEngine();
    await loadCatalog();
    selectAll(catalogStore.themes.map((t) => t.id));
  });
  return <AppShell />;
}
