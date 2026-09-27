/* @refresh reload */
import { render } from "solid-js/web";

import "./index.css";

import App from "./App";

if (import.meta.env.MODE === "production") {
  document.addEventListener("contextmenu", (event) => event.preventDefault());
}

render(() => <App />, document.getElementById("root") as HTMLElement);
