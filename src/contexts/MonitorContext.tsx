import { createContext, type ParentProps, useContext } from "solid-js";
import { useMonitorSelection, useMonitorThemeSync } from "~/hooks/monitor";

interface MonitorItem {
  value: string;
  label: string;
}

interface MonitorContext {
  id: Accessor<string>;
  setId: Setter<string>;
  list: Accessor<MonitorItem[]>;
  specificThemes: Accessor<[string, string][]>;
  allSameTheme: Accessor<boolean>;
  handleChange: (value: string) => void;
}

const MonitorContext = createContext<MonitorContext>();

export const MonitorProvider = (props: ParentProps) => {
  const {
    monitorID,
    setMonitorID,
    monitors,
    monitorSpecificThemes,
    monitorSpecificThemesIsSame,
    handleMonitorChange,
  } = useMonitorSelection();

  useMonitorThemeSync(monitorID, monitorSpecificThemesIsSame);

  return (
    <MonitorContext.Provider
      value={{
        id: monitorID,
        setId: setMonitorID,
        list: monitors,
        specificThemes: monitorSpecificThemes,
        allSameTheme: monitorSpecificThemesIsSame,
        handleChange: handleMonitorChange,
      }}
    >
      {props.children}
    </MonitorContext.Provider>
  );
};

export const useMonitor = () => {
  const context = useContext(MonitorContext);
  if (!context) {
    throw new Error("useMonitor: 必须在MonitorProvider内部使用");
  }
  return context;
};
