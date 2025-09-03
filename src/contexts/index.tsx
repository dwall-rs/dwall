import type { ParentProps } from "solid-js";

import { TranslationsProvider } from "./TranslationsContext";
import { ConfigProvider } from "./ConfigContext";
import { ThemeProvider } from "./ThemeContext";
import { MonitorProvider } from "./MonitorContext";
import { UpdateProvider } from "./UpdateContext";
import { SettingsProvider } from "./SettingsContext";
import { TaskProvider } from "./TaskContext";
import { ToastProvider, TooltipProvider } from "fluent-solid/lib/index";

export const AppProvider = (props: ParentProps) => {
  return (
    <ToastProvider>
      <TooltipProvider>
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
      </TooltipProvider>
    </ToastProvider>
  );
};

export { useTranslations } from "./TranslationsContext";
export { useConfig } from "./ConfigContext";
export { useTheme } from "./ThemeContext";
export { useMonitor } from "./MonitorContext";
export { useSettings } from "./SettingsContext";
export { useUpdate } from "./UpdateContext";
export { useTask } from "./TaskContext";
export { useToast } from "fluent-solid";
