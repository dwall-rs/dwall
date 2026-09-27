// Responsibility: apply the appearance intent to data-theme; skip the transition when the value is unchanged, swallow AbortError, and keep StrictMode noise out.
import { themeStore, setResolved } from "@/store/theme.store";
import { createEffect } from "solid-js";
import type { ResolvedTheme } from "~/domain/types";

export function useThemeEngine() {
  let lastApplied: ResolvedTheme | null = null;

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
