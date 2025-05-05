import { createContext, useContext, type ParentProps } from "solid-js";

import { useConfigState } from "~/hooks/state";

interface ConfigContext {
  data: Resource<Config>;
  refetch: () => Config | Promise<Config | undefined> | null | undefined;
  mutate: Setter<Config | undefined>;
}

const ConfigContext = createContext<ConfigContext>();

export const ConfigProvider = (props: ParentProps) => {
  const config = useConfigState();

  return (
    <ConfigContext.Provider value={config}>
      {props.children}
    </ConfigContext.Provider>
  );
};

export const useConfig = () => {
  const context = useContext(ConfigContext);
  if (!context) {
    throw new Error("useConfig: must be used within a ConfigProvider");
  }
  return context;
};
