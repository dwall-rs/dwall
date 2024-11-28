import { children, createMemo, createSignal, onMount, Show } from "solid-js";
import {
  applyTheme,
  checkThemeExists,
  closeLastThemeTask,
  getAppliedThemeID,
  readConfigFile,
  showMainWindow,
} from "./commands";
import { LazyButton, LazyFlex, LazySpace, LazyTooltip } from "./lazy";
import ImageCarousel from "./components/ImageCarousel";
import "./App.scss";
import { useDark } from "alley-components";
import { AiFillSetting } from "solid-icons/ai";
import Settings from "./components/Settings";
import { AppContext } from "./context";

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

interface ThemeItem {
  id: string;
  thumbnail: string[];
}

const useThemeSelector = (themes: ThemeItem[]) => {
  const [config, setConfig] = createSignal<Config>();
  const [appliedThemeID, setAppliedThemeID] = createSignal<string>();
  const [index, setIndex] = createSignal(0);
  const [themeButtonStatus, setThemeButtonStatus] = createSignal({
    exists: false,
    applied: false,
  });

  const currentTheme = createMemo(() => themes[index()]);

  const autoRun = async (config: Config) => {
    const { selected_theme_id, ...themeParams } = config;
    if (!selected_theme_id) return;

    await applyTheme({
      selected_theme_id,
      ...themeParams,
    });

    setAppliedThemeID(selected_theme_id);
    setIndex(themes.findIndex((t) => t.id === selected_theme_id));
  };

  const onMenuItemClick = async (idx: number) => {
    setIndex(idx);
    try {
      await checkThemeExists(themes[idx].id);
      setThemeButtonStatus({ exists: true, applied: false });
    } catch (e) {
      setThemeButtonStatus({ exists: false, applied: false });
    }
  };

  const onCloseTask = async () => {
    closeLastThemeTask();
    setConfig((prev) => {
      const stoppedConfig = { ...prev!, selected_theme_id: undefined };
      applyTheme(stoppedConfig);
      return stoppedConfig;
    });
    setAppliedThemeID();
  };

  const onApply = async () => {
    const newConfig = {
      ...config()!,
      selected_theme_id: currentTheme().id,
    };
    await applyTheme(newConfig);
    setConfig(newConfig);
    setAppliedThemeID(newConfig.selected_theme_id);
  };

  return {
    config,
    setConfig,
    appliedThemeID,
    setAppliedThemeID,
    index,
    setIndex,
    themeButtonStatus,
    currentTheme,
    autoRun,
    onMenuItemClick,
    onCloseTask,
    onApply,
  };
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
    setConfig,
    appliedThemeID,
    setAppliedThemeID,
    index,
    setIndex,
    themeButtonStatus,
    currentTheme,
    autoRun,
    onMenuItemClick,
    onCloseTask,
    onApply,
  } = useThemeSelector(themes);

  useDark();

  onMount(async () => {
    await showMainWindow();

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

  const menu = children(() =>
    themes.map((item, idx) => (
      <div
        onClick={() => onMenuItemClick(idx)}
        classList={{
          "menu-item": true,
          active: idx === index(),
          applied: item.id === appliedThemeID(),
        }}
      >
        <LazyTooltip
          placement="right"
          text={
            appliedThemeID() === item.id ? `${item.id}（正在使用）` : item.id
          }
          delay={500}
          showArrow
        >
          <img src={item.thumbnail[0]} alt={item.id} width={64} />
        </LazyTooltip>
      </div>
    )),
  );

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
          <LazyFlex direction="vertical" gap={8} class="menu">
            {menu()}
          </LazyFlex>

          <LazyTooltip placement="right" text="设置" delay={500} showArrow>
            <LazyButton
              icon={<AiFillSetting />}
              onClick={() => setShowSettings(true)}
            />
          </LazyTooltip>
        </LazyFlex>

        <LazyFlex direction="vertical" gap={8} justify="center" align="center">
          <ImageCarousel
            images={currentTheme().thumbnail.map((src) => ({
              src,
              alt: currentTheme().id,
            }))}
            height="480px"
            width="480px"
          />

          <LazySpace gap={8}>
            <LazyButton type="primary" disabled={themeButtonStatus().exists}>
              下载
            </LazyButton>
            <Show
              when={appliedThemeID() !== currentTheme().id}
              fallback={
                <LazyButton onClick={onCloseTask} danger>
                  停止
                </LazyButton>
              }
            >
              <LazyButton
                type="primary"
                disabled={!themeButtonStatus().exists}
                onClick={onApply}
              >
                应用
              </LazyButton>
            </Show>
          </LazySpace>
        </LazyFlex>
      </LazyFlex>

      <Settings />
    </AppContext.Provider>
  );
};

export default App;
