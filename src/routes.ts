import { lazy } from "solid-js";
import App from "./App";

const routes = [
  { path: "/", component: App },
  { path: "/settings", component: lazy(() => import("./layout/settings")) },
  { path: "/theme/:id", component: lazy(() => import("./layout/theme")) },
];

export default routes;
