import { createResource } from "solid-js";
import { AiFillGithub } from "solid-icons/ai";

import { getVersion } from "@tauri-apps/api/app";
import { ask, message } from "@tauri-apps/plugin-dialog";
import { open } from "@tauri-apps/plugin-shell";

import { openLogDir } from "~/commands";
import { useUpdate } from "~/contexts";
import { Button } from "~/components/button";
import { Label } from "~/components/label";
import {
  Tooltip,
  TooltipArrow,
  TooltipContent,
  TooltipTrigger,
} from "~/components/tooltip";
import { t } from "~/i18n";

const SettingsFooter = () => {
  const [version] = createResource(getVersion);
  const { update: resource, recheckUpdate } = useUpdate();

  const onOpenLogDir = async () => {
    await openLogDir();
  };

  const onUpdate = async () => {
    if (!resource()) {
      recheckUpdate();
    }
    const update = resource();
    if (!update) {
      if (update === null) await message(t("settings.message.isLatestVersion"));
      return;
    }

    const { currentVersion, version, body } = update;

    const result = await ask(
      `Current version ${currentVersion}, new version ${version} available.\n\nChangelog:\n\n${body}\n\nUpdate now?`,
      "Dwall",
    );
    if (!result) return;
  };

  const onOpenGithub = () => open("https://github.com/dwall-rs/dwall");

  return (
    <div class="w-full flex items-center justify-evenly">
      <div class="flex items-center space-x-2">
        <Label class="font-light">{t("settings.label.version")}</Label>
        <Tooltip>
          <TooltipTrigger>
            <Button onClick={onUpdate} variant="ghost">
              {version()}
            </Button>
          </TooltipTrigger>
          <TooltipContent side="top">
            {t("settings.tooltip.checkForNewVersion")}
            <TooltipArrow />
          </TooltipContent>
        </Tooltip>
      </div>

      <div class="flex items-center space-x-2">
        <Label class="font-light">{t("settings.label.sourceCode")}</Label>

        <Button
          onClick={onOpenGithub}
          variant="ghost"
          icon={{ icon: <AiFillGithub />, ariaLabel: "Github" }}
        />
      </div>

      <Button variant="ghost" onClick={onOpenLogDir}>
        {t("settings.button.openLogDirectory")}
      </Button>
    </div>
  );
};

export default SettingsFooter;
