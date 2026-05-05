import { useColorMode } from "~/hooks/useColorMode";
import useDark from "~/hooks/useDark";
import { useAppInitialization } from "~/hooks/useAppInitialization";

import { SidebarProvider } from "~/components/sidebar";
import { Toaster } from "~/components/toast";

import Titlebar from "./titlebar";
import AppSidebar from "./sidebar";
import { route, type ThemeRoute } from "~/router";
import { Match, Switch } from "solid-js";
import Settings from "./settings";
import Theme from "./theme";
import { AppProvider } from "~/contexts";

const Layout = () => {
  useDark();
  useColorMode();

  useAppInitialization();

  return (
    <AppProvider>
      <Titlebar />
      <SidebarProvider>
        <AppSidebar />
        <main class="flex-1 pt-10 px-3 flex flex-col items-center justify-center">
          <Switch>
            <Match when={route().path === "settings"}>
              <Settings />
            </Match>
            <Match when={route().path === "theme"}>
              <Theme id={(route() as ThemeRoute).id} />
            </Match>
          </Switch>
          {/*{props.children}*/}
        </main>
      </SidebarProvider>
      <Toaster />
    </AppProvider>
  );
};

export default Layout;
