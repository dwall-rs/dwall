import { onMount } from "solid-js";
import { showMainWindow } from "./commands";

const App = () => {
  onMount(() => {
    showMainWindow();
  });
};

export default App;
