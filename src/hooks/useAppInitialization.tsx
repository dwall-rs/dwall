import { onMount } from "solid-js";
import {
  getAppliedThemeID,
  setTitlebarColorMode,
  showWindow,
} from "~/commands";
import { useMonitor, useTheme } from "~/contexts";
import { themes } from "~/themes";
import { detectColorMode } from "~/utils/color";

/**
 * App initialization Hook, used to handle application startup logic
 * @param menuItemIndex Current menu item index
 * @param handleThemeSelection Function to handle theme selection
 */
export const useAppInitialization = (
  menuItemIndex: Accessor<number | undefined>,
  handleThemeSelection: (index: number) => void,
) => {
  const { setMenuItemIndex, setAppliedThemeID } = useTheme();
  const { id: monitorID } = useMonitor();

  onMount(async () => {
    await setTitlebarColorMode(detectColorMode());

    if (!import.meta.env.PROD) await showWindow("main");

    const mii = menuItemIndex();
    if (mii !== undefined) handleThemeSelection(mii);

    const applied_theme_id = await getAppliedThemeID(monitorID());
    if (applied_theme_id) {
      const themeIndex = themes.findIndex((t) => t.id === applied_theme_id);
      if (themeIndex !== -1) {
        setMenuItemIndex(themeIndex);
        handleThemeSelection(themeIndex);
        setAppliedThemeID(applied_theme_id);
        return;
      }
    }
  });
};
