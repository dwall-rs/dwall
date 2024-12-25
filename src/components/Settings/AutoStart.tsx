import { LazySwitch } from "~/lazy";
import SettingsItem from "./item";
import { createSignal, onMount } from "solid-js";
import { checkAutoStart, disableAutoStart, enableAutoStart } from "~/commands";
import { message } from "@tauri-apps/plugin-dialog";
import { useAppContext } from "~/context";
import { translate } from "~/utils/i18n";

const AutoStart = () => {
  const { translations } = useAppContext();
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
          translate(translations()!, "message-disable-startup-failed", {
            error: String(e),
          }),
        );
        return;
      }
    } else {
      try {
        await enableAutoStart();
      } catch (e) {
        message(
          translate(translations()!, "message-startup-failed", {
            error: String(e),
          }),
        );
        return;
      }
    }
    setAutoStartState((prev) => !prev);
  };

  return (
    <SettingsItem label={translate(translations()!, "label-launch-at-startup")}>
      <LazySwitch checked={autoStartState()} onChange={onSwitchAutoStart} />
    </SettingsItem>
  );
};

export default AutoStart;
