import { createSignal } from "solid-js";

export const useSettingsState = () => {
  const [showSettings, setShowSettings] = createSignal(false);

  return {
    showSettings,
    setShowSettings,
  };
};
