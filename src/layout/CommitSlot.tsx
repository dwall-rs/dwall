// 职责：顶栏右侧提交槽——位置恒定，按模式渲染 FixedCommit 或 RandomCommit。
import { uiStore } from "@/store/ui.store";
import { FixedCommit } from "./FixedCommit";
import { RandomCommit } from "./RandomCommit";

export function CommitSlot() {
  return uiStore.mode === "fixed" ? <FixedCommit /> : <RandomCommit />;
}
