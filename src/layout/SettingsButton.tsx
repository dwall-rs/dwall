// 职责：设置入口；在设置视图时高亮（active）。
import { uiStore, toggleView } from "@/store/ui.store";
import { Button } from "~/components/ui/button";
import { Settings } from "lucide-solid";

export function SettingsButton() {
  return (
    <Button
      onClick={toggleView}
      title="设置"
      variant={uiStore.view === "settings" ? "primary" : "outline"}
    >
      <Settings class="size-4" />
    </Button>
  );
}
