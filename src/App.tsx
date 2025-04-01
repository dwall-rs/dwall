import { Show } from "solid-js";

import useDark from "alley-components/lib/hooks/useDark";

import { LazyFlex } from "~/lazy";

import { ThemeMenu } from "./components/ThemeMenu";
import Settings from "./components/Settings";
import SidebarButtons from "./components/SidebarButtons";
import ThemeShowcase from "./components/ThemeShowcase";
import Updater from "./components/Update";
import Select from "./components/Select";

import { useColorMode } from "./hooks/useColorMode";
import { useAppInitialization } from "./hooks/useAppInitialization";

import { useMonitor, useTheme, useTranslations, useSettings } from "~/contexts";

import { themes } from "./themes";

import "./App.scss";

const App = () => {
  const { translate } = useTranslations();
  const theme = useTheme();

  const { currentTheme, downloadThemeID, menuItemIndex, handleThemeSelection } =
    theme;

  const {
    list: monitors,
    handleChange: handleMonitorChange,
    id: monitorID,
  } = useMonitor();
  // const { handleClosure: handleTaskClosure } = task;
  const { showSettings } = useSettings();

  useDark();
  useColorMode();

  useAppInitialization(menuItemIndex, handleThemeSelection);

  return (
    <>
      <LazyFlex class="app" align="center">
        <LazyFlex
          direction="vertical"
          align="center"
          justify="between"
          class="sidebar"
        >
          <ThemeMenu themes={themes} />

          <SidebarButtons />
        </LazyFlex>

        <Show when={!showSettings() && currentTheme()} fallback={<Settings />}>
          <LazyFlex direction="vertical" gap={16} align="center">
            <Select
              options={monitors()}
              placeholder={translate("label-select-monitor")}
              onChange={handleMonitorChange}
              value={monitorID()}
              label={translate("label-select-monitor")}
              disabled={!!downloadThemeID()} // 下载主题时禁用选择框
            />

            <ThemeShowcase />
          </LazyFlex>
        </Show>
      </LazyFlex>

      <Updater />
    </>
  );
};

export default App;
