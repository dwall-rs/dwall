import { createSignal, onMount } from "solid-js";

import { message } from "@tauri-apps/plugin-dialog";

import { Switch } from "~/components/switch";
import SettingsItem from "./SettingsItem";

import { checkAutoStart, disableAutoStart, enableAutoStart } from "~/commands";
import { t } from "~/i18n";

const AutoStart = () => {
  const id = "auto-start-switch";
  const [autoStartState, setAutoStartState] = createSignal(false);

  onMount(async () => {
    const state = await checkAutoStart();
    setAutoStartState(state);
  });

  const onSwitchAutoStart = async () => {
    if (autoStartState()) {
      try {
        await disableAutoStart();
      } catch (e) {
        message(
          t("settings.message.disableStartupFailed", { error: String(e) }),
          {
            kind: "error",
          },
        );
        return;
      }
    } else {
      try {
        await enableAutoStart();
      } catch (e) {
        message(t("settings.message.startupFailed", { error: String(e) }), {
          kind: "error",
        });
        return;
      }
    }
    setAutoStartState((prev) => !prev);
  };

  return (
    <SettingsItem
      // label={translate("label-launch-at-startup")}
      label={t("settings.label.launchAtStartup")}
      help={t("settings.help.launchAtStartup")}
      for={id}
    >
      <Switch
        id={id}
        checked={autoStartState()}
        onCheckedChange={onSwitchAutoStart}
      />
    </SettingsItem>
  );
};

export default AutoStart;
