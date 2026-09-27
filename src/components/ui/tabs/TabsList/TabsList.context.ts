import { createContext, useContext } from "solid-js";

interface TabsListContextValue {
  /** Whether focusing activates the tab (base-ui semantics, default false: focus only highlights, click activates) */
  activateOnFocus: boolean;
}

export const TabsListContext = createContext<TabsListContextValue>();

export const useTabsListContext = () => {
  const context = useContext(TabsListContext);
  if (!context)
    throw new Error(
      "useTabsListContext must be used within a <TabsList> component",
    );
  return context;
};
