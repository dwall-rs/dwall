import { createContext, useContext } from "solid-js";

interface TabsListContextValue {
  activateOnFocus: boolean;
}

export const TabsListContext = createContext<TabsListContextValue>();

export const useTabsListContext = () => {
  const context = useContext(TabsListContext);
  if (!context) throw new Error("useTabsListContext 必须用在 <TabsList> 内部");
  return context;
};
