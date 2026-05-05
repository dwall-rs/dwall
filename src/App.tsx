import { Match, Switch } from "solid-js";

import { useColorMode } from "~/hooks/useColorMode";
import useDark from "~/hooks/useDark";
import { useAppInitialization } from "~/hooks/useAppInitialization";
import { useFontLoader } from "./hooks/useFontLoader";

import { SidebarProvider } from "~/components/sidebar";
import { Toaster } from "~/components/toast";

import { route, type ThemeRoute } from "~/router";

import AppSidebar from "~/layout/sidebar";
import Settings from "~/layout/settings";
import Theme from "~/layout/theme";

const App = () => {
  useFontLoader();
  useDark();
  useColorMode();

  useAppInitialization();

  return (
    <>
      <SidebarProvider>
        <AppSidebar />
        <main class="flex-1 pt-3 flex flex-col items-center justify-center bg-neutral-50 dark:bg-neutral-900 h-screen overflow-hidden">
          <Switch>
            <Match when={route().path === "settings"}>
              <Settings />
            </Match>
            <Match when={route().path === "theme"}>
              <Theme id={(route() as ThemeRoute).id} />
            </Match>
          </Switch>
        </main>
      </SidebarProvider>
      <Toaster />
    </>
  );
};

export default App;
