import { createResource, createSignal } from "solid-js";
import { AiFillGithub } from "solid-icons/ai";

import { getVersion } from "@tauri-apps/api/app";
import { message } from "@tauri-apps/plugin-dialog";
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

  const [checking, setChecking] = createSignal(false);

  const onOpenLogDir = async () => {
    await openLogDir();
  };

  const onUpdate = async () => {
    if (resource() === undefined) {
      // Force recheck update to ensure `resource()` cannot be undefined
      try {
        setChecking(true);
        await recheckUpdate();
      } finally {
        setChecking(false);
      }
    }

    const update = resource();

    if (!update) {
      await message(t("settings.message.isLatestVersion"));
      return;
    }

    const { currentVersion, version } = update;

    const msg = t("common.message.updateAvailable", {
      version,
      currentVersion,
    });

    message(msg, "Dwall");
  };

  const onOpenGithub = () => open("https://github.com/dwall-rs/dwall");

  return (
    <div class="w-full flex items-center justify-evenly">
      <div class="flex items-center space-x-2">
        <Label class="font-light">{t("settings.label.version")}</Label>
        <Tooltip>
          <TooltipTrigger>
            <Button onClick={onUpdate} variant="ghost" disabled={checking()}>
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
