// Responsibility: the window shell — one persistent sidebar, one scrollable work
// surface, global navigation shortcuts, and the overlays that belong to the window
// rather than to a page.
import { Match, Switch, createEffect, onCleanup, onMount } from "solid-js";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Toaster } from "@/components/ui/toast";
import { useThemeEngine } from "@/hooks/useThemeEngine";
import { showsTitlebarBorder } from "@/store/ui.selectors";
import { setView, uiStore } from "@/store/ui.store";
import { clsx } from "@/utils";
import { Sidebar } from "./Sidebar";
import { SettingsSaveBar } from "./SettingsSaveBar";
import { WallpaperView } from "@/views/WallpaperView";
import { ThemesView } from "@/views/ThemesView";
import { DisplaysView } from "@/views/DisplaysView";
import { SettingsView } from "@/views/SettingsView";

export function AppShell() {
  useThemeEngine();
  let main: HTMLElement | undefined;

  // The shell owns one scroll container, so a surface would otherwise open at the
  // scroll offset the previous one left behind.
  createEffect(() => {
    uiStore.view;
    requestAnimationFrame(() => {
      if (main) main.scrollTop = 0;
    });
  });

  onMount(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const mod = event.ctrlKey || event.metaKey;

      if (mod && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setView("themes");
        requestAnimationFrame(() =>
          document.getElementById("theme-search")?.focus(),
        );
        return;
      }

      if (mod && event.key === ",") {
        event.preventDefault();
        setView("settings");
        return;
      }

      // Alt + 1..4: the rail in keyboard form. Kept off Ctrl so it cannot collide with
      // browser shortcuts the webview may still handle.
      if (event.altKey && ["1", "2", "3", "4"].includes(event.key)) {
        event.preventDefault();
        setView(
          (["wallpaper", "themes", "displays", "settings"] as const)[
            Number(event.key) - 1
          ],
        );
      }
    };

    document.addEventListener("keydown", onKeyDown);
    onCleanup(() => document.removeEventListener("keydown", onKeyDown));
  });

  return (
    <TooltipProvider delay={300}>
      <div
        class={clsx(
          "relative z-10 flex h-screen overflow-hidden bg-background",
          showsTitlebarBorder() && "border-t border-border",
        )}
      >
        <Sidebar />
        <main
          ref={main}
          data-scroll-root
          class="relative min-w-0 flex-1 overflow-y-auto"
        >
          <div class="min-h-full">
            <Switch>
              <Match when={uiStore.view === "wallpaper"}>
                <WallpaperView />
              </Match>
              <Match when={uiStore.view === "themes"}>
                <ThemesView />
              </Match>
              <Match when={uiStore.view === "displays"}>
                <DisplaysView />
              </Match>
              <Match when={uiStore.view === "settings"}>
                <SettingsView />
              </Match>
            </Switch>
          </div>
        </main>
      </div>
      <SettingsSaveBar />
      <Toaster position="bottom-right" />
    </TooltipProvider>
  );
}
