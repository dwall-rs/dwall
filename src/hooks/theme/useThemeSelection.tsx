import { createMemo } from "solid-js";
import { validateTheme } from "~/commands";

/**
 * Theme selection management Hook, used to handle theme selection related functions
 * @param themes List of available themes
 * @param config Application configuration
 * @param menuItemIndex Currently selected theme index
 * @param setMenuItemIndex Function to set the current selected theme index
 * @param setThemeExists Function to set whether the theme exists
 * @returns Theme selection related states and methods
 */
export const useThemeSelection = (
  themes: ThemeItem[],
  config: () => Config | undefined,
  menuItemIndex: () => number | undefined,
  setMenuItemIndex: (index: number) => void,
  setThemeExists: (exists: boolean) => void,
) => {
  // Calculate current selected theme
  const currentTheme = createMemo(() => {
    const idx = menuItemIndex();
    if (idx === undefined) return;
    return themes[idx];
  });

  // Handle theme selection
  const handleThemeSelection = async (idx: number) => {
    setMenuItemIndex(idx);
    try {
      await validateTheme(config()?.themes_directory ?? "", themes[idx].id);
      setThemeExists(true);
    } catch (e) {
      setThemeExists(false);
      console.error("Failed to check theme existence:", e);
    }
  };

  return {
    currentTheme,
    handleThemeSelection,
  };
};
