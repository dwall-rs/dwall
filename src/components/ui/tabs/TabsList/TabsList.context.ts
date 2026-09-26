import { createContext, useContext } from "solid-js";

interface TabsListContextValue {
  /** 聚焦是否激活 tab(base-ui 语义,默认 false:聚焦只高亮,点击才激活) */
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
