import { createContext, type ParentProps, useContext } from "solid-js";
import { useSettingsState } from "~/hooks/state/useSettingsState";

interface SettingsContext {
  showSettings: Accessor<boolean>;
  setShowSettings: Setter<boolean>;
}

const SettingsContext = createContext<SettingsContext>();

export const SettingsProvider = (props: ParentProps) => {
  const settings = useSettingsState();

  return (
    <SettingsContext.Provider value={settings}>
      {props.children}
    </SettingsContext.Provider>
  );
};

export const useSettings = () => {
  const context = useContext(SettingsContext);
  if (!context) {
    throw new Error("useSettings: 必须在SettingsProvider内部使用");
  }
  return context;
};
