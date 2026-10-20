// Responsibility: App shell; mounts the global toaster and the optional title-bar separator. Everything else unchanged.
import { useThemeEngine } from "~/hooks/useThemeEngine";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Toaster } from "@/components/ui/toast";
import { showsTitlebarBorder } from "@/store/ui.selectors";
import { clsx } from "@/utils";
import { GlobalBar } from "./GlobalBar";
import { Body } from "~/views/Body";

export function AppShell() {
  useThemeEngine();
  return (
    <TooltipProvider delay={300}>
      {/* The window's title bar is painted with the app's own background colour, so it is
          invisible when the content sits directly below it. This border marks where it
          ends; it is only inside the box (Tailwind's preflight is border-box), so it does
          not add a pixel of overflow. */}
      <div
        class={clsx(
          "relative z-10 flex h-screen flex-col overflow-hidden",
          showsTitlebarBorder() && "border-t border-border",
        )}
      >
        <GlobalBar />
        <Body />
      </div>
      {/* Failures of engine/theme/config actions surface here; without a mounted
          toaster every rejected IPC call was invisible to the user. */}
      <Toaster position="bottom-right" />
    </TooltipProvider>
  );
}
