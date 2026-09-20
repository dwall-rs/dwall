// 职责：根组件，仅渲染外壳；不持有任何业务状态（状态在各 store）。
import { AppShell } from "@/layout/AppShell";
import { onMount } from "solid-js";
import { load } from "./store/settings.store";

export default function App() {
  onMount(() => {
    load();
  });
  return <AppShell />;
}
