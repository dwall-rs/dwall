import {
  createContext,
  createSignal,
  type ParentProps,
  useContext,
} from "solid-js";
import type { ThemeID } from "~/themes";

type Route = { path: "settings" } | { path: "theme"; id: ThemeID };

interface RouteContextValue {
  route: Accessor<Route>;
  navigate: (path: Route) => void;
}

const RouteContext = createContext<RouteContextValue>();

export const RouteProvider = (props: ParentProps) => {
  const [route, setRoute] = createSignal<Route>({
    path: "theme",
    id: "Big Sur",
  });

  const navigate = (path: Route) => {
    setRoute(path);
  };

  return (
    <RouteContext.Provider value={{ route, navigate }}>
      {props.children}
    </RouteContext.Provider>
  );
};

export const useRoute = () => {
  const context = useContext(RouteContext);
  if (!context) throw new Error("useRoute must be used within a RouteContext");
  return context;
};
