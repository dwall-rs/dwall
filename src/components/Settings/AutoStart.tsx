import { LazySwitch } from "~/lazy";
import SettingsItem from "./item";
import { createSignal, onMount } from "solid-js";
import { checkAutoStart, disableAutoStart, enableAutoStart } from "~/commands";
import { AiOutlineCheck, AiOutlineClose } from "solid-icons/ai";

const AutoStart = () => {
  const [autoStartState, setAutoStartState] = createSignal(false);

  onMount(async () => {
    const state = await checkAutoStart();
    setAutoStartState(state);
  });

  const onSwitchAutoStart = async () => {
    if (autoStartState()) {
      await disableAutoStart();
    } else {
      await enableAutoStart();
    }
    setAutoStartState((prev) => !prev);
  };

  return (
    <SettingsItem label="开机自启">
      <LazySwitch
        checked={autoStartState()}
        setChecked={onSwitchAutoStart}
        checkedChild={<AiOutlineCheck />}
        uncheckedChild={<AiOutlineClose />}
      />
    </SettingsItem>
  );
};

export default AutoStart;
