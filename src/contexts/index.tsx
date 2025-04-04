import type { ParentProps } from "solid-js";

import { TranslationsProvider } from "./TranslationsContext";
import { ConfigProvider } from "./ConfigContext";
import { ThemeProvider } from "./ThemeContext";
import { MonitorProvider } from "./MonitorContext";
import { UpdateProvider } from "./UpdateContext";
import { SettingsProvider } from "./SettingsContext";
import { TaskProvider } from "./TaskContext";

export const AppProvider = (props: ParentProps) => {
  return (
    <TranslationsProvider>
      <ConfigProvider>
        <ThemeProvider>
          <MonitorProvider>
            <SettingsProvider>
              <UpdateProvider>
                <TaskProvider>{props.children}</TaskProvider>
              </UpdateProvider>
            </SettingsProvider>
          </MonitorProvider>
        </ThemeProvider>
      </ConfigProvider>
    </TranslationsProvider>
  );
};

export { useTranslations } from "./TranslationsContext";
export { useConfig } from "./ConfigContext";
export { useTheme } from "./ThemeContext";
export { useMonitor } from "./MonitorContext";
export { useSettings } from "./SettingsContext";
export { useUpdate } from "./UpdateContext";
export { useTask } from "./TaskContext";
