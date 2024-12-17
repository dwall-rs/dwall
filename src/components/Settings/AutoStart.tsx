import { LazySwitch } from "~/lazy";
import SettingsItem from "./item";
import { createSignal, onMount } from "solid-js";
import { checkAutoStart, disableAutoStart, enableAutoStart } from "~/commands";
import { message } from "@tauri-apps/plugin-dialog";

const AutoStart = () => {
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
        message(`关闭开机启动失败：\n${e}`);
        return;
      }
    } else {
      try {
        await enableAutoStart();
      } catch (e) {
        message(`开机启动失败：\n${e}`);
        return;
      }
    }
    setAutoStartState((prev) => !prev);
  };

  return (
    <SettingsItem label="开机自启">
      <LazySwitch checked={autoStartState()} onChange={onSwitchAutoStart} />
    </SettingsItem>
  );
};

export default AutoStart;
