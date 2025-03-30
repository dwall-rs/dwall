import { Show } from "solid-js";

import useDark from "alley-components/lib/hooks/useDark";

import { LazyFlex } from "~/lazy";

import { ThemeMenu } from "./components/ThemeMenu";
import Settings from "./components/Settings";
import SidebarButtons from "./components/SidebarButtons";
import { useThemeSelector } from "./hooks/useThemeManager";
import ThemeShowcase from "./components/ThemeShowcase";
import Updater from "./components/Update";
import Select from "./components/Select";

import { useColorMode } from "./hooks/useColorMode";
import { useUpdateManager } from "./hooks/useUpdateManager";
import { useAppInitialization } from "./hooks/useAppInitialization";

import { AppContext } from "./context";

import { themes } from "./themes";

import "./App.scss";
import { useTranslations } from "./components/TranslationsContext";

const App = () => {
  const { translate } = useTranslations();
  const themeManager = useThemeSelector(themes);

  // 解构主题管理器中的各个部分
  const {
    theme,
    config: configManager,
    monitor,
    task,
    settings,
    update,
  } = themeManager;

  // 从各部分中提取需要的状态和方法
  const {
    currentTheme,
    appliedThemeID,
    downloadThemeID,
    setDownloadThemeID,
    menuItemIndex,
    setMenuItemIndex,
    themeExists,
    handleThemeSelection,
    handleThemeApplication,
    setAppliedThemeID,
  } = theme;

  const { data: config, refetch: refetchConfig } = configManager;
  const {
    list: monitors,
    handleChange: handleMonitorChange,
    id: monitorID,
  } = monitor;
  const { handleClosure: handleTaskClosure } = task;
  const { show: showSettings, setShow: setShowSettings } = settings;
  const { data: updateData, recheck: recheckUpdate } = update;

  // 使用更新管理Hook
  const { showUpdateDialog, setShowUpdateDialog } = useUpdateManager(
    updateData,
    recheckUpdate,
  );

  useDark();
  useColorMode();

  useAppInitialization(menuItemIndex, handleThemeSelection);

  return (
    <AppContext.Provider
      value={{
        update: {
          resource: updateData,
          refetch: recheckUpdate,
          showDialog: showUpdateDialog,
          setShowDialog: setShowUpdateDialog,
        },
        config,
        refetchConfig,
        settings: { show: showSettings, setShow: setShowSettings },
        theme: {
          currentTheme,
          appliedThemeID,
          setAppliedThemeID,
          downloadThemeID,
          setDownloadThemeID,
          menuItemIndex,
          setMenuItemIndex,
          themeExists,
          handleThemeSelection,
          handleThemeApplication,
        },
        monitor: {
          list: monitors,
          handleChange: handleMonitorChange,
          id: monitorID,
        },
        task: {
          handleClosure: handleTaskClosure,
        },
      }}
    >
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
    </AppContext.Provider>
  );
};

export default App;
