/* ===== src/components/views/Body.tsx ===== */
// 职责：按 view 在主视图/设置视图间切换（cross-fade）。
import { uiStore } from "@/store/ui.store";
import { MainView } from "./MainView";
import { SettingsView } from "./SettingsView";
import { clsx } from "@/utils";

export function Body() {
  return (
    <div class="relative min-h-0 flex-1">
      <div
        class={clsx(
          "absolute inset-0",
          uiStore.view === "main" ? "animate-fade-up" : "hidden",
        )}
      >
        <MainView />
      </div>
      <div
        class={clsx(
          "absolute inset-0 overflow-y-auto",
          uiStore.view === "settings" ? "animate-fade-up" : "hidden",
        )}
      >
        <SettingsView />
      </div>
    </div>
  );
}
