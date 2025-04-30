import { createSignal, onMount } from "solid-js";

import { message } from "@tauri-apps/plugin-dialog";

import SettingsItem from "./Item";
import { LazySwitch } from "~/lazy";

import { checkAutoStart, disableAutoStart, enableAutoStart } from "~/commands";

import { useTranslations } from "~/contexts";

const AutoStart = () => {
  const { translate, translateErrorMessage } = useTranslations();
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
        message(translateErrorMessage("message-disable-startup-failed", e), {
          kind: "error",
        });
        return;
      }
    } else {
      try {
        await enableAutoStart();
      } catch (e) {
        message(translateErrorMessage("message-startup-failed", e), {
          kind: "error",
        });
        return;
      }
    }
    setAutoStartState((prev) => !prev);
  };

  return (
    <SettingsItem
      label={translate("label-launch-at-startup")}
      help={translate("help-launch-at-startup")}
    >
      <LazySwitch checked={autoStartState()} onChange={onSwitchAutoStart} />
    </SettingsItem>
  );
};

export default AutoStart;
