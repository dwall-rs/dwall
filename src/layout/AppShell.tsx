// Responsibility: App shell; mounts the minimum-size notice. Everything else unchanged.
import { useThemeEngine } from "~/hooks/useThemeEngine";
import { TooltipProvider } from "@/components/ui/tooltip";
import { GlobalBar } from "./GlobalBar";
import { Body } from "~/views/Body";

export function AppShell() {
  useThemeEngine();
  return (
    <TooltipProvider delay={300}>
      <div class="relative z-10 flex h-screen flex-col overflow-hidden">
        <GlobalBar />
        <Body />
      </div>
    </TooltipProvider>
  );
}
