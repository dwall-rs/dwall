import { createContext, type ParentProps, useContext } from "solid-js";
import { useThemeApplication } from "~/hooks/theme/useThemeApplication";
import { useThemeState } from "~/hooks/theme/useThemeState";
import { useLocationPermission } from "~/hooks/useLocationPermission";
import { useConfig } from "./ConfigContext";

interface ThemeContext {
  appliedThemeID: Accessor<string | undefined>;
  setAppliedThemeID: Setter<string | undefined>;
  downloadingTheme: Accessor<boolean>;
  setDownloadingTheme: Setter<boolean>;
  handleThemeApplication: (
    monitorID: Accessor<string>,
    themeID?: string,
  ) => Promise<void>;
}

const ThemeContext = createContext<ThemeContext>();

export const ThemeProvider = (props: ParentProps) => {
  const { data: config, mutate, refetch: refetchConfig } = useConfig();
  const themeState = useThemeState();
  const {
    appliedThemeID,
    setAppliedThemeID,
    downloadingTheme,
    setDownloadingTheme,
  } = themeState;

  // Use location permission Hook
  const { checkLocationPermission } = useLocationPermission(mutate);

  // Use theme application Hook
  const { handleThemeApplication } = useThemeApplication(
    config,
    refetchConfig,
    checkLocationPermission,
    setAppliedThemeID,
  );

  return (
    <ThemeContext.Provider
      value={{
        appliedThemeID,
        setAppliedThemeID,
        downloadingTheme,
        setDownloadingTheme,
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
