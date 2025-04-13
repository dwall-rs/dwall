import { Show } from "solid-js";

import { LazyFlex } from "~/lazy";

import { ThemeMenu } from "./components/ThemeMenu";
import Settings from "./components/Settings";
import SidebarButtons from "./components/SidebarButtons";
import ThemeShowcase from "./components/ThemeShowcase";
import Updater from "./components/Update";
import Select from "./components/Select";

import useDark from "~/hooks/useDark";
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
  const { showSettings } = useSettings();

  useDark();
  useColorMode();

  useAppInitialization(translate, menuItemIndex, handleThemeSelection);

  return (
    <>
      <LazyFlex class="app" align="center" gap="l" justify="stretch">
        <LazyFlex
          direction="column"
          align="center"
          justify="between"
          class="sidebar"
        >
          <ThemeMenu themes={themes} />

          <SidebarButtons />
        </LazyFlex>

        <Show when={!showSettings() && currentTheme()} fallback={<Settings />}>
          <LazyFlex direction="column" gap="l" align="center">
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
