// 职责：外壳；挂载最小尺寸提示。其余不变。
import { useThemeEngine } from "~/hooks/useThemeEngine";
import { GlobalBar } from "./GlobalBar";
import { Body } from "~/views/Body";

export function AppShell() {
  useThemeEngine();
  return (
    <div class="relative z-10 flex h-screen flex-col overflow-hidden">
      <GlobalBar />
      <Body />
    </div>
  );
}
