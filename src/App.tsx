import { Show } from "solid-js";

import { LazyFlex } from "~/lazy";

import Settings from "./components/Settings";
import ThemeShowcase from "./components/ThemeShowcase";
import Updater from "./components/Update";
import Select from "./components/Select";
import Sidebar from "./components/Sidebar";

import useDark from "~/hooks/useDark";
import { useColorMode } from "./hooks/useColorMode";
import { useAppInitialization } from "./hooks/useAppInitialization";

import { useMonitor, useTheme, useTranslations, useSettings } from "~/contexts";

import * as styles from "./App.css";

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
      <LazyFlex class={styles.app} align="center" gap="l" justify="stretch">
        <Sidebar />

        <Show when={!showSettings() && currentTheme()} fallback={<Settings />}>
          <LazyFlex direction="column" gap="l" align="center">
            <Select
              options={monitors()}
              placeholder={translate("label-select-monitor")}
              onChange={handleMonitorChange}
              value={monitorID()}
              label={translate("label-select-monitor")}
              disabled={!!downloadThemeID()} // Disable select box when downloading theme
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
