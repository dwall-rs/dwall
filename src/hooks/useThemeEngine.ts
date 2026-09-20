// 职责：把外观意图落到 data-theme；值未变不启动过渡，吞掉 AbortError，消除 StrictMode 噪音。
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

      // 目标值没变就不启动过渡：既省一次无意义动画，也避免 StrictMode 双跑触发 AbortError
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
          // 过渡被后续切换打断是无害的，吞掉 AbortError，别让它变成未捕获报错
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
