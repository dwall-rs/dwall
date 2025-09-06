import { open } from "@tauri-apps/plugin-shell";
import { createEffect, onMount } from "solid-js";
import { toastMessageLinkLikeButton } from "~/App.css";
import {
  getAppliedThemeID,
  setTitlebarColorMode,
  showWindow,
} from "~/commands";
import { useMonitor, useTheme, useToast, useUpdate } from "~/contexts";
import type { Translate } from "~/contexts/TranslationsContext";
import { themes } from "~/themes";
import { detectColorMode } from "~/utils/color";

/**
 * App initialization Hook, used to handle application startup logic
 * @param menuItemIndex Current menu item index
 * @param handleThemeSelection Function to handle theme selection
 */
export const useAppInitialization = (
  translate: Translate,
  menuItemIndex: Accessor<number | undefined>,
  handleThemeSelection: (index: number) => void,
) => {
  const { setMenuItemIndex, setAppliedThemeID } = useTheme();
  const { id: monitorID } = useMonitor();
  const toast = useToast();
  const { update } = useUpdate();

  const openGithubRepository = async () => {
    await open("https://github.com/dwall-rs/dwall");
  };

  const githubStarMessage = (
    <span>
      {translate("message-github-star")}
      <button
        type="button"
        class={toastMessageLinkLikeButton}
        onClick={openGithubRepository}
      >
        dwall
      </button>
    </span>
  );

  const updateMessage = (
    <span>
      {translate("message-update-available", {
        version: update()?.version ?? "",
        currentVersion: update()?.currentVersion ?? "",
      })}
    </span>
  );

  createEffect(() => {
    if (update()) {
      toast.info(updateMessage, {
        position: "top-right",
      });
    }
  });

  onMount(async () => {
    await setTitlebarColorMode(detectColorMode());

    if (import.meta.env.PROD) await showWindow("main");

    const mii = menuItemIndex();
    if (mii !== undefined) handleThemeSelection(mii);

    toast.info(githubStarMessage, {
      position: "top-right",
      duration: 5000,
    });

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
