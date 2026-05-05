import type { ParentProps } from "solid-js";

import { ConfigProvider } from "./ConfigContext";
import { ThemeProvider } from "./ThemeContext";
import { MonitorProvider } from "./MonitorContext";
import { UpdateProvider } from "./UpdateContext";
import { SettingsProvider } from "./SettingsContext";
import { TaskProvider } from "./TaskContext";

export const AppProvider = (props: ParentProps) => {
  return (
    <ConfigProvider>
      <MonitorProvider>
        <ThemeProvider>
          <SettingsProvider>
            <UpdateProvider>
              <TaskProvider>{props.children}</TaskProvider>
            </UpdateProvider>
          </SettingsProvider>
        </ThemeProvider>
      </MonitorProvider>
    </ConfigProvider>
  );
};

export { useConfig } from "./ConfigContext";
export { useTheme } from "./ThemeContext";
export { useMonitor } from "./MonitorContext";
export { useSettings } from "./SettingsContext";
export { useUpdate } from "./UpdateContext";
export { useTask } from "./TaskContext";
