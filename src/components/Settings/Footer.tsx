import { createResource } from "solid-js";
import { AiFillGithub } from "solid-icons/ai";

import { getVersion } from "@tauri-apps/api/app";
import { ask, message } from "@tauri-apps/plugin-dialog";
import { open } from "@tauri-apps/plugin-shell";

import { openLogDir } from "~/commands";
import { useTranslations, useUpdate } from "~/contexts";
import {
  LazyButton,
  LazyFlex,
  LazyLabel,
  LazySpace,
  LazyTooltip,
} from "~/lazy";

const SettingsFooter = () => {
  const [version] = createResource(getVersion);
  const { translate } = useTranslations();
  const { update: resource, recheckUpdate, setShowUpdateDialog } = useUpdate();

  const onOpenLogDir = async () => {
    await openLogDir();
  };

  const onUpdate = async () => {
    if (!resource()) {
      recheckUpdate();
    }
    const update = resource();
    if (!update) {
      if (update === null)
        await message(translate("message-version-is-the-latest"));
      return;
    }

    const { currentVersion, version, body } = update;

    const result = await ask(
      `Current version ${currentVersion}, new version ${version} available.\n\nChangelog:\n\n${body}\n\nUpdate now?`,
      "Dwall",
    );
    if (!result) return;
    setShowUpdateDialog(true);
  };

  const onOpenGithub = () => open("https://github.com/dwall-rs/dwall");

  return (
    <LazyFlex justify="between">
      <LazySpace gap="s">
        <LazyLabel>{translate("label-version")}</LazyLabel>

        <LazyTooltip
          content={translate("tooltip-check-new-version")}
          relationship="label"
          withArrow
        >
          <LazyButton
            appearance="transparent"
            style={{ "min-width": "48px" }}
            onClick={onUpdate}
          >
            {version()}
          </LazyButton>
        </LazyTooltip>
      </LazySpace>

      <LazySpace gap="s">
        <LazyLabel>{translate("label-source-code")}</LazyLabel>

        <LazyButton
          appearance="subtle"
          icon={<AiFillGithub />}
          onClick={onOpenGithub}
        />
      </LazySpace>

      <LazySpace>
        <LazyButton appearance="subtle" onClick={onOpenLogDir}>
          {translate("button-open-log-directory")}
        </LazyButton>
      </LazySpace>
    </LazyFlex>
  );
};

export default SettingsFooter;
