import { createSignal, createEffect, onMount } from "solid-js";
import type { Accessor, Setter } from "solid-js";

type DarkMode = "dark" | "light" | "system";

/**
 * A custom hook that manages the dark mode state based on user preference or system setting.
 * It returns an accessor to the current dark mode state and a setter to change it.
 *
 * @param {DarkMode} [mode="system"] - The initial mode to start with, either 'dark', 'light', or 'system'.
 * @returns {[Accessor<boolean>, Setter<boolean>]} A tuple where the first element is an accessor that returns whether dark mode is active,
 * and the second element is a setter that can be used to manually toggle dark mode.
 */
const useDark = (
  mode: DarkMode = "system",
): [Accessor<boolean>, Setter<boolean>] => {
  const [isDark, setIsDark] = createSignal(mode === "dark");

  onMount(() => {
    if (mode !== "system") return;

    if (matchMedia("(prefers-color-scheme: dark)").matches) {
      setIsDark(true);
    } else {
      setIsDark(false);
    }

    window
      .matchMedia("(prefers-color-scheme: dark)")
      .addEventListener("change", (event) => {
        if (event.matches) {
          setIsDark(true);
        } else {
          setIsDark(false);
        }
      });
  });

  createEffect(() => {
    if (isDark()) window.document.documentElement.setAttribute("class", "dark");
    else window.document.documentElement.removeAttribute("class");
  });

  return [isDark, setIsDark];
};

export default useDark;