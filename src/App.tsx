import { createSignal, onMount, Show } from "solid-js";
import { LazyFlex, LazyTooltip, LazyButton, LazyBadge } from "~/lazy";
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

const App = () => {
  const [showSettings, setShowSettings] = createSignal(false);

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
    onMenuItemClick,
    onCloseTask,
    onApply,
    setAppliedThemeID,
    update,
    recheckUpdate,
  } = useThemeSelector(themes);

  useDark();

  onMount(async () => {
    await setTitlebarColorMode(detectColorMode());

    await showWindow("main");

    const mii = menuItemIndex();
    if (mii !== undefined) onMenuItemClick(mii);

    const applied_theme_id = await getAppliedThemeID();
    if (applied_theme_id) {
      const themeIndex = themes.findIndex((t) => t.id === applied_theme_id);
      if (themeIndex !== -1) {
        setMenuItemIndex(themeIndex);
        onMenuItemClick(themeIndex);
        setAppliedThemeID(applied_theme_id);
        return;
      }
    }
  });

  return (
    <AppContext.Provider
      value={{
        update: { resource: update, refetch: recheckUpdate },
        config,
        refetchConfig,
        settings: { show: showSettings, setShow: setShowSettings },
      }}
    >
      <LazyFlex
        style={{ height: "100vh", flex: 1, padding: "20px 0" }}
        justify="round"
        align="center"
      >
        <LazyFlex
          direction="vertical"
          align="center"
          justify="between"
          style={{ height: "100%" }}
        >
          <ThemeMenu
            themes={themes}
            index={menuItemIndex()}
            appliedThemeID={appliedThemeID()}
            onMenuItemClick={(idx) => {
              setShowSettings(false);
              onMenuItemClick(idx);
            }}
          />

          <div style={{ position: "relative" }}>
            <LazyTooltip
              positioning="after"
              content="设置"
              relationship="label"
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
            <Show when={update()}>
              <LazyTooltip content="检测到新版本" relationship="label">
                <LazyBadge
                  style={{ position: "absolute", right: "4px", top: "4px" }}
                  size="extra-small"
                  color="severe"
                />
              </LazyTooltip>
            </Show>
          </div>
        </LazyFlex>

        <Show when={!showSettings() && currentTheme()} fallback={<Settings />}>
          <ThemeShowcase
            currentTheme={currentTheme()!}
            themeExists={themeExists}
            appliedThemeID={appliedThemeID}
            downloadThemeID={downloadThemeID}
            setDownloadThemeID={setDownloadThemeID}
            onDownload={() => setDownloadThemeID(currentTheme()!.id)}
            onApply={onApply}
            onCloseTask={onCloseTask}
            onMenuItemClick={onMenuItemClick}
            index={menuItemIndex()!}
          />
        </Show>
      </LazyFlex>
    </AppContext.Provider>
  );
};

export default App;
