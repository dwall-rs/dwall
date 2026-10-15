// Responsibility: App shell; mounts the global toaster. Everything else unchanged.
import { useThemeEngine } from "~/hooks/useThemeEngine";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Toaster } from "@/components/ui/toast";
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
      {/* Failures of engine/theme/config actions surface here; without a mounted
          toaster every rejected IPC call was invisible to the user. */}
      <Toaster position="bottom-right" />
    </TooltipProvider>
  );
}
