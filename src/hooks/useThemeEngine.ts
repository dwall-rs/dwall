// Responsibility: apply the appearance intent to data-theme; skip the transition when the value is unchanged, swallow AbortError, and keep StrictMode noise out. Also matches the window's title bar to that appearance.
import { themeStore, setResolved } from "@/store/theme.store";
import { settingsStore } from "@/store/settings.store";
import { setTitlebarColorMode } from "@/ipc";
import type { ColorScheme } from "@/ipc";
import { logger } from "@/utils";
import { createEffect } from "solid-js";
import type { ResolvedTheme } from "~/models/types";

const log = logger.child("theme");

export function useThemeEngine() {
  let lastApplied: ResolvedTheme | null = null;

  // The title bar belongs to the appearance: Dwall's own background colour, or the one the
  // Windows theme asks for. Waiting for the config keeps a first paint from being undone a
  // moment later by the stored preference.
  createEffect(() => {
    const config = settingsStore.config;
    if (!config) return;

    const colorMode: ColorScheme =
      themeStore.resolved === "dark" ? "DARK" : "LIGHT";

    // Windows 11 matches the colour; on Windows 10 only the light/dark side follows, which
    // the backend handles.
    void setTitlebarColorMode(
      colorMode,
      config.title_bar_color_follows_windows_theme,
    ).catch((error) => log.warn("Failed to set the title bar colour", error));
  });

  createEffect(() => {
    const mql = window.matchMedia("(prefers-color-scheme: dark)");

    const apply = () => {
      const resolved =
        themeStore.mode === "system"
          ? mql.matches
            ? "dark"
            : "light"
          : themeStore.mode;

      // Skip the transition when the target value is unchanged: saves a pointless animation and avoids AbortError from StrictMode's double run
      if (lastApplied === resolved) {
        setResolved(resolved);
        return;
      }
      lastApplied = resolved;

      const run = () => {
        document.documentElement.setAttribute("data-theme", resolved);
        setResolved(resolved);
      };

      const doc = document as Document & {
        startViewTransition?: (cb: () => void) => { finished: Promise<void> };
      };
      if (doc.startViewTransition) {
        try {
          // A transition interrupted by a later switch is harmless; swallow AbortError instead of letting it become an uncaught error
          doc.startViewTransition(run).finished.catch(() => {});
        } catch {
          run();
        }
      } else {
        run();
      }
    };

    apply();
    mql.addEventListener("change", apply);
    return () => mql.removeEventListener("change", apply);
  });
}
