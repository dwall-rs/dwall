import { createEffect, createSignal, onCleanup } from "solid-js";
import type { Theme } from "../types";

export function useTheme(theme: () => Theme | "system") {
  const getSystemTheme = (): Theme =>
    typeof window !== "undefined" &&
    window.matchMedia?.("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";

  const [actualTheme, setActualTheme] = createSignal<Theme>(
    theme() !== "system" ? (theme() as Theme) : getSystemTheme(),
  );

  createEffect(() => {
    if (theme() !== "system") {
      setActualTheme(theme() as Theme);
      return;
    }

    setActualTheme(getSystemTheme());

    if (typeof window === "undefined") return;
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = (e: MediaQueryListEvent) =>
      setActualTheme(e.matches ? "dark" : "light");

    try {
      mq.addEventListener("change", handler);
      onCleanup(() => mq.removeEventListener("change", handler));
    } catch {
      // Safari < 14 fallback
      mq.addListener(handler);
      onCleanup(() => mq.removeListener(handler as any));
    }
  });

  return actualTheme;
}
