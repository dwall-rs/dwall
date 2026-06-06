import { createSignal } from "solid-js";
import type { ThemeID } from "~/themes";

export type SettingsRoute = { path: "settings" };
export type ThemeRoute = { path: "theme"; id: string };
export type Route = SettingsRoute | ThemeRoute;

const [route, setRoute] = createSignal<Route>({
  path: "theme",
  id: "Big Sur",
});

const navigate = (r: Route) => {
  setRoute(r);
};

const navigateToTheme = (id: ThemeID) => {
  setRoute({ path: "theme", id });
};

export { route, navigate, navigateToTheme };
