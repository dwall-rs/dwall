/* @refresh reload */
import { render } from "solid-js/web";
import "alley-components/lib/index.css";
import "fluent-solid/lib/index.css";
import "./index.scss";
import App from "./App";

if (import.meta.env.MODE === "production") {
  document.addEventListener("contextmenu", (event) => event.preventDefault());
}

render(() => <App />, document.getElementById("root") as HTMLElement);
