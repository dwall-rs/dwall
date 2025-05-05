import {
  createContext,
  createMemo,
  type ParentProps,
  useContext,
} from "solid-js";
import { checkThemeExists } from "~/commands";
import { useThemeApplication } from "~/hooks/theme/useThemeApplication";
import { useThemeState } from "~/hooks/theme/useThemeState";
import { useLocationPermission } from "~/hooks/useLocationPermission";
import { themes } from "~/themes";
import { useConfig } from "./ConfigContext";

interface ThemeContext {
  currentTheme: Accessor<ThemeItem | undefined>;
  appliedThemeID: Accessor<string | undefined>;
  setAppliedThemeID: Setter<string | undefined>;
  downloadThemeID: Accessor<string | undefined>;
  setDownloadThemeID: Setter<string | undefined>;
  menuItemIndex: Accessor<number | undefined>;
  setMenuItemIndex: Setter<number | undefined>;
  themeExists: Accessor<boolean>;
  handleThemeSelection: (index: number) => void;
  handleThemeApplication: (monitorID: Accessor<string>) => Promise<void>;
}

const ThemeContext = createContext<ThemeContext>();

export const ThemeProvider = (props: ParentProps) => {
  const { data: config, mutate, refetch: refetchConfig } = useConfig();
  const themeState = useThemeState();
  const {
    appliedThemeID,
    setAppliedThemeID,
    downloadThemeID,
    setDownloadThemeID,
    menuItemIndex,
    setMenuItemIndex,
    themeExists,
    setThemeExists,
    setShowSettings,
  } = themeState;

  const currentTheme = createMemo(() => {
    const idx = menuItemIndex();
    if (idx === undefined) return;
    return themes[idx];
  });

  // Handle theme selection
  const handleThemeSelection = async (idx: number) => {
    setMenuItemIndex(idx);
    try {
      await checkThemeExists(config()?.themes_directory ?? "", themes[idx].id);
      setThemeExists(true);
    } catch (e) {
      setThemeExists(false);
      console.error(`Failed to check theme existence: index=${idx} error=${e}`);
    }
  };

  // Use location permission Hook
  const { checkLocationPermission } = useLocationPermission(
    mutate,
    setShowSettings,
  );

  // Use theme application Hook
  const { handleThemeApplication } = useThemeApplication(
    config,
    refetchConfig,
    currentTheme,
    checkLocationPermission,
    setAppliedThemeID,
  );

  return (
    <ThemeContext.Provider
      value={{
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
      }}
    >
      {props.children}
    </ThemeContext.Provider>
  );
};

export const useTheme = () => {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error("useTheme: must be used within a ThemeProvider");
  }
  return context;
};
