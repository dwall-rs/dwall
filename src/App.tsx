import { createSignal, onMount, Show } from "solid-js";
import { LazyFlex, LazyTooltip, LazyButton, LazySpace } from "~/lazy";
import { AiFillSetting } from "solid-icons/ai";
import useDark from "alley-components/lib/hooks/useDark";

import { ThemeMenu } from "./components/ThemeMenu";
import Settings from "./components/Settings";
import { AppContext } from "./context";
import {
  showWindow,
  getAppliedThemeID,
  setTitlebarColorMode,
} from "~/commands";
import { useThemeSelector } from "./components/ThemeContext";
import "./App.scss";
import ThemeShowcase from "./components/ThemeShowcase";
import { detectColorMode } from "./utils/color";
import { themes } from "./themes";
import { TbArrowBigUpLinesFilled } from "solid-icons/tb";
import Updater from "./components/Update";
import { translate } from "./utils/i18n";
import Select from "./components/Select";

const App = () => {
  const [showUpdateDialog, setShowUpdateDialog] = createSignal<boolean>();

  const {
    config,
    refetchConfig,
    appliedThemeID,
    downloadThemeID,
    setDownloadThemeID,
    menuItemIndex,
    setMenuItemIndex,
    themeExists,
    currentTheme,
    handleThemeSelection,
    handleTaskClosure,
    handleThemeApplication,
    setAppliedThemeID,
    update,
    recheckUpdate,
    showSettings,
    setShowSettings,
    translations,
    monitors,
    handleMonitorChange,
    monitorID,
  } = useThemeSelector(themes);

  useDark();

  onMount(async () => {
    await setTitlebarColorMode(detectColorMode());

    if (!import.meta.env.PRODEV) await showWindow("main");

    const mii = menuItemIndex();
    if (mii !== undefined) handleThemeSelection(mii);

    const applied_theme_id = await getAppliedThemeID();
    if (applied_theme_id) {
      const themeIndex = themes.findIndex((t) => t.id === applied_theme_id);
      if (themeIndex !== -1) {
        setMenuItemIndex(themeIndex);
        handleThemeSelection(themeIndex);
        setAppliedThemeID(applied_theme_id);
        return;
      }
    }
  });

  const onUpdate = () => {
    update() && setShowUpdateDialog(true);
  };

  return (
    <AppContext.Provider
      value={{
        update: {
          resource: update,
          refetch: recheckUpdate,
          showDialog: showUpdateDialog,
          setShowDialog: setShowUpdateDialog,
        },
        config,
        refetchConfig,
        settings: { show: showSettings, setShow: setShowSettings },
        translations,
      }}
    >
      <LazyFlex class="app" align="center">
        <LazyFlex
          direction="vertical"
          align="center"
          justify="between"
          class="sidebar"
        >
          <ThemeMenu
            themes={themes}
            index={menuItemIndex()}
            appliedThemeID={appliedThemeID()}
            onMenuItemClick={(idx) => {
              setShowSettings(false);
              handleThemeSelection(idx);
            }}
          />

          <LazySpace
            direction="vertical"
            gap={8}
            justify="end"
            align="center"
            class="sidebar-buttons"
          >
            <Show when={update()}>
              <LazyTooltip
                positioning="after"
                content={translate(
                  translations()!,
                  "tooltip-new-version-available",
                )}
                relationship="label"
                withArrow
              >
                <LazyButton
                  appearance="transparent"
                  shape="circular"
                  icon={<TbArrowBigUpLinesFilled />}
                  onClick={onUpdate}
                />
              </LazyTooltip>
            </Show>

            <LazyTooltip
              positioning="after"
              content={translate(translations()!, "tooltip-settings")}
              relationship="label"
              withArrow
            >
              <LazyButton
                appearance="transparent"
                shape="circular"
                icon={<AiFillSetting />}
                onClick={() => {
                  setMenuItemIndex();
                  setShowSettings(true);
                }}
              />
            </LazyTooltip>
          </LazySpace>
        </LazyFlex>

        <Show when={!showSettings() && currentTheme()} fallback={<Settings />}>
          <LazyFlex direction="vertical" gap={16} align="center">
            <Select
              options={monitors()}
              placeholder="选择显示器"
              onChange={handleMonitorChange}
              value={monitorID()}
              label="选择显示器"
            />

            <ThemeShowcase
              currentTheme={currentTheme()!}
              themeExists={themeExists}
              appliedThemeID={appliedThemeID}
              downloadThemeID={downloadThemeID}
              setDownloadThemeID={setDownloadThemeID}
              onDownload={() => setDownloadThemeID(currentTheme()!.id)}
              onApply={handleThemeApplication}
              onCloseTask={handleTaskClosure}
              onMenuItemClick={handleThemeSelection}
              index={menuItemIndex()!}
            />
          </LazyFlex>
        </Show>
      </LazyFlex>

      <Updater />
    </AppContext.Provider>
  );
};

export default App;
