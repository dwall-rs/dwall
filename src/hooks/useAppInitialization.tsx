import { onMount } from "solid-js";
import { setTitlebarColorMode, showWindow } from "~/commands";
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
  onMount(async () => {
    await setTitlebarColorMode(detectColorMode());

    if (!import.meta.env.PROD) await showWindow("main");

    const mii = menuItemIndex();
    if (mii !== undefined) handleThemeSelection(mii);
  });
};
