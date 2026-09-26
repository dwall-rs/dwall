// 职责：顶栏右侧提交槽——按「配置里实际生效的模式」渲染，而非仅看编辑模式。
import { settingsStore } from "@/store/settings.store";
import { FixedCommit } from "./FixedCommit";
import { RandomCommit } from "./RandomCommit";

export function CommitSlot() {
  const random = () => settingsStore.config?.wallpaper_mode?.mode === "random";
  return random() ? <RandomCommit /> : <FixedCommit />;
}
