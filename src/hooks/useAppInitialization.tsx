import { createEffect, onMount } from "solid-js";
import { open } from "@tauri-apps/plugin-shell";

import {
  getAppliedThemeID,
  readConfigFile,
  setTitlebarColorMode,
  showWindow,
} from "~/commands";
import { toast } from "~/components/toast";
import { useMonitor, useTheme, useUpdate } from "~/contexts";
import { detectColorMode } from "~/utils/color";
import { themes } from "~/themes";
import { useMonitorThemeSync } from "./monitor";
import { navigateToTheme } from "~/router";
import { t } from "~/i18n";
import { Button } from "~/components/button";

/**
 * App initialization Hook, used to handle application startup logic
 */
export const useAppInitialization = () => {
  const { setAppliedThemeID } = useTheme();
  const { id: monitorID, allSameTheme } = useMonitor();
  const { update } = useUpdate();

  useMonitorThemeSync(monitorID, allSameTheme);

  const openGithubRepository = async () => {
    await open("https://github.com/dwall-rs/dwall");
  };

  const githubStarMessage = (
    <span>
      {t("common.message.githubStar")}
      <Button variant="link" size="sm" onClick={openGithubRepository}>
        dwall
      </Button>
    </span>
  );

  const updateMessage = (u: Update) => (
    <span>
      {t("common.message.updateAvailable", {
        version: u.version,
        currentVersion: u.currentVersion,
      })}
    </span>
  );

  createEffect(() => {
    const u = update();
    if (u) {
      toast.info(updateMessage(u), {
        position: "top-right",
      });
    }
  });

  onMount(async () => {
    const config = await readConfigFile();

    if (!config.title_bar_color_follows_windows_theme)
      await setTitlebarColorMode(detectColorMode());

    if (import.meta.env.PROD) await showWindow("main");

    toast.info(githubStarMessage, {
      position: "top-right",
      duration: 5000,
    });

    const appliedThemeID = await getAppliedThemeID(monitorID());
    if (appliedThemeID) {
      setAppliedThemeID(appliedThemeID);
      navigateToTheme(appliedThemeID);
    } else {
      navigateToTheme(themes[0].id);
    }
  });
};
