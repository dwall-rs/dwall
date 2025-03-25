import { onMount } from "solid-js";
import { setTitlebarColorMode } from "~/commands";

/**
 * System color mode management Hook, used to monitor system color mode changes and update the title bar
 */
export const useColorMode = () => {
  onMount(() => {
    const darkModeMediaQuery = window.matchMedia(
      "(prefers-color-scheme: dark)",
    );

    const handleColorSchemeChange = (event: MediaQueryListEvent) => {
      setTitlebarColorMode(event.matches ? "DARK" : "LIGHT");
    };

    darkModeMediaQuery.addEventListener("change", handleColorSchemeChange);
    return () =>
      darkModeMediaQuery.removeEventListener("change", handleColorSchemeChange);
  });
};
