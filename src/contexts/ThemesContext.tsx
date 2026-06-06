import { createContext, useContext, type ParentProps } from "solid-js";
import { useThemes } from "~/hooks/theme/useThemes";
import type { ThemeItem } from "~/themes";
import type { CustomizedTheme } from "~/types";

interface ThemesContext {
  allThemes: Accessor<(CustomizedTheme | ThemeItem)[]>;
}

const ThemesContext = createContext<ThemesContext>();

export const ThemesProvider = (props: ParentProps) => {
  const { allThemes } = useThemes();

  return (
    <ThemesContext.Provider value={{ allThemes }}>
      {props.children}
    </ThemesContext.Provider>
  );
};

export const useThemesContext = () => {
  const context = useContext(ThemesContext);
  if (!context) {
    throw new Error("useThemesContext: must be used within a ThemesProvider");
  }
  return context;
};
