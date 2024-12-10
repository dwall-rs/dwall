import { createSignal, onMount, Show } from "solid-js";
import { LazyFlex, LazyTooltip, LazyButton } from "~/lazy";
import { AiFillSetting } from "solid-icons/ai";
import { useDark } from "alley-components";

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

// 图片导入逻辑
const images = {
  Catalina: Object.values(
    import.meta.glob("~/assets/thumbnail/Catalina/*.avif", {
      import: "default",
      eager: true,
    }) as Record<string, string>,
  ),
  "Big Sur": Object.values(
    import.meta.glob("~/assets/thumbnail/BigSur/*.avif", {
      import: "default",
      eager: true,
    }) as Record<string, string>,
  ),
  Mojave: Object.values(
    import.meta.glob("~/assets/thumbnail/Mojave/*.avif", {
      import: "default",
      eager: true,
    }) as Record<string, string>,
  ),
};

const App = () => {
  const themes: ThemeItem[] = Object.entries(images).map(
    ([id, thumbnails]) => ({
      id,
      thumbnail: thumbnails,
    }),
  );

  const [showSettings, setShowSettings] = createSignal(false);

  const {
    config,
    refetchConfig,
    appliedThemeID,
    downloadThemeID,
    setDownloadThemeID,
    index,
    setIndex,
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

    onMenuItemClick(index());

    const applied_theme_id = await getAppliedThemeID();
    if (applied_theme_id) {
      const themeIndex = themes.findIndex((t) => t.id === applied_theme_id);
      if (themeIndex !== -1) {
        setIndex(themeIndex);
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
            index={index()}
            appliedThemeID={appliedThemeID()}
            onMenuItemClick={(idx) => {
              setShowSettings(false);
              onMenuItemClick(idx);
            }}
          />

          <LazyTooltip placement="right" text="设置" delay={500} showArrow>
            <LazyButton
              appearance="transparent"
              shape="circular"
              icon={<AiFillSetting />}
              onClick={() => {
                // setIndex(-1);
                setShowSettings(true);
              }}
            />
          </LazyTooltip>
        </LazyFlex>

        <Show when={!showSettings()} fallback={<Settings />}>
          <ThemeShowcase
            currentTheme={currentTheme}
            themeExists={themeExists}
            appliedThemeID={appliedThemeID}
            downloadThemeID={downloadThemeID}
            setDownloadThemeID={setDownloadThemeID}
            onDownload={() => setDownloadThemeID(currentTheme().id)}
            onApply={onApply}
            onCloseTask={onCloseTask}
            onMenuItemClick={onMenuItemClick}
            index={index}
          />
        </Show>
      </LazyFlex>
    </AppContext.Provider>
  );
};

export default App;
