import { createSignal, onMount } from "solid-js";
import { Show } from "solid-js";
import { LazyFlex, LazyTooltip, LazyButton } from "~/lazy";
import { AiFillSetting } from "solid-icons/ai";
import { useDark } from "alley-components";

import { ThemeMenu } from "./components/ThemeMenu";
import { ThemeActions } from "./components/ThemeActions";
import ImageCarousel from "./components/ImageCarousel";
import Download from "./components/Download";
import Settings from "./components/Settings";
import { AppContext } from "./context";
import { showWindow, getAppliedThemeID, readConfigFile } from "~/commands";
import { useThemeSelector } from "./components/ThemeContext";
import "./App.scss";

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
    appliedThemeID,
    downloadThemeID,
    setDownloadThemeID,
    index,
    setIndex,
    themeExists,
    currentTheme,
    autoRun,
    onMenuItemClick,
    onCloseTask,
    onApply,
    setConfig,
    setAppliedThemeID,
  } = useThemeSelector(themes);

  useDark();

  onMount(async () => {
    await showWindow("main");

    const configData = await readConfigFile();
    setConfig(configData);

    onMenuItemClick(index());

    const applied_theme_id = await getAppliedThemeID();
    if (applied_theme_id) {
      const themeIndex = themes.findIndex((t) => t.id === applied_theme_id);
      if (themeIndex !== -1) {
        setIndex(themeIndex);
        setAppliedThemeID(applied_theme_id);
        return;
      }
    }

    autoRun(configData);
  });

  return (
    <AppContext.Provider
      value={{
        config,
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
            onMenuItemClick={onMenuItemClick}
          />

          <LazyTooltip placement="right" text="设置" delay={500} showArrow>
            <LazyButton
              icon={<AiFillSetting />}
              onClick={() => setShowSettings(true)}
            />
          </LazyTooltip>
        </LazyFlex>

        <LazyFlex
          direction="vertical"
          gap={16}
          justify="center"
          align="center"
          style={{ position: "relative" }}
        >
          <ImageCarousel
            images={currentTheme().thumbnail.map((src) => ({
              src,
              alt: currentTheme().id,
            }))}
            height="480px"
            width="480px"
          />

          <ThemeActions
            themeExists={themeExists()}
            appliedThemeID={appliedThemeID()}
            currentThemeID={currentTheme().id}
            onDownload={() => setDownloadThemeID(currentTheme().id)}
            onApply={onApply}
            onCloseTask={onCloseTask}
            downloadThemeID={downloadThemeID()}
          />

          <Show when={downloadThemeID()}>
            <Download
              themeID={downloadThemeID()!}
              onFinished={() => {
                setDownloadThemeID();
                onMenuItemClick(index());
              }}
            />
          </Show>
        </LazyFlex>
      </LazyFlex>

      <Settings />
    </AppContext.Provider>
  );
};

export default App;
