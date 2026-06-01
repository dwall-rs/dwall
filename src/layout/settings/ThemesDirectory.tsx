import { createSignal } from "solid-js";

import { confirm, message, open } from "@tauri-apps/plugin-dialog";

import { Button } from "~/components/button";
import {
  Tooltip,
  TooltipArrow,
  TooltipContent,
  TooltipTrigger,
} from "~/components/tooltip";

import SettingsItem from "./SettingsItem";

import { moveDirectory, openDir, writeConfigFile } from "~/commands";

import { useConfig } from "~/contexts";
import { t } from "~/i18n";

const ThemesDirectory = () => {
  const { data: config, refetch: refetchConfig } = useConfig();

  const [path, setPath] = createSignal(config()?.themes_directory);

  const onOpenThemesDirectory = () => {
    openDir(path()!);
  };

  const onChangePath = async () => {
    const dirPath = await open({ directory: true });
    if (!dirPath) return;

    const newThemesDirectory = `${dirPath}\\themes`;

    const ok = await confirm(
      t("settings.message.changeThemesDirectory", {
        directory: newThemesDirectory,
      }),
    );
    if (!ok) return;

    try {
      await moveDirectory(config()!.themes_directory, newThemesDirectory);
      await writeConfigFile({
        ...config()!,
        themes_directory: newThemesDirectory,
      });

      message(
        t("settings.message.movedThemesDirectory", {
          directory: newThemesDirectory,
        }),
      );
      setPath(newThemesDirectory);
      refetchConfig();
    } catch (e) {
      message(String(e), { kind: "error" });
    }
  };

  return (
    <SettingsItem
      orientation="vertical"
      label={t("settings.label.themesDirectory")}
    >
      <div class="flex items-center justify-between">
        <Tooltip>
          <TooltipTrigger>
            <Button size="sm" onClick={onOpenThemesDirectory} variant="ghost">
              {path()}
            </Button>
          </TooltipTrigger>
          <TooltipContent>
            <TooltipArrow />
            <p>{t("settings.tooltip.openThemesDirectory")}</p>
          </TooltipContent>
        </Tooltip>

        <Button size="sm" onClick={onChangePath} variant="outline">
          {t("settings.button.selectDirectory")}
        </Button>
      </div>
    </SettingsItem>
  );
};

export default ThemesDirectory;
