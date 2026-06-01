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
import { toast } from "~/components/toast";

const CustomizedThemesDirectory = () => {
  const { data: config, refetch: refetchConfig } = useConfig();

  const [path, setPath] = createSignal(config()?.customized_themes_directory);

  const onOpenThemesDirectory = () => {
    openDir(path()!);
  };

  const onChangePath = async () => {
    const dirPath = await open({ directory: true });
    if (!dirPath) return;

    const newCustomizedThemesDirectory = `${dirPath}\\customize`;

    const ok = await confirm(
      t("settings.message.changeCustomizedThemesDirectory", {
        directory: newCustomizedThemesDirectory,
      }),
    );
    if (!ok) return;

    try {
      await moveDirectory(
        config()!.customized_themes_directory,
        newCustomizedThemesDirectory,
      );
      await writeConfigFile({
        ...config()!,
        customized_themes_directory: newCustomizedThemesDirectory,
      });

      message(
        t("settings.message.movedCustomizedThemesDirectory", {
          directory: newCustomizedThemesDirectory,
        }),
      );
      setPath(newCustomizedThemesDirectory);
      refetchConfig();
    } catch (e) {
      message(String(e), { kind: "error" });
    }
  };

  const handleOpenDwallMaker = () => {
    toast.warning("This app is under development, please be patient.");
  };

  return (
    <SettingsItem
      orientation="vertical"
      label={t("settings.label.customizedThemesDirectory")}
      help={
        <span>
          {t("settings.help.customizedThemesDirectory")}
          <Button size="xs" variant="link" onClick={handleOpenDwallMaker}>
            Dwall Maker
          </Button>
        </span>
      }
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
            <p>{t("settings.tooltip.openCustomizedThemesDirectory")}</p>
          </TooltipContent>
        </Tooltip>

        <Button size="sm" onClick={onChangePath} variant="outline">
          {t("settings.button.selectDirectory")}
        </Button>
      </div>
    </SettingsItem>
  );
};

export default CustomizedThemesDirectory;
