import { themes } from "~/themes";
import { LazyFlex } from "~/lazy";
import ThemeMenu from "~/components/ThemeMenu";
import SidebarButtons from "./SidebarButtons";
import styles from "./index.module.scss";

const Sidebar = () => (
  <LazyFlex
    direction="column"
    align="center"
    justify="between"
    class={styles.sidebar}
  >
    <ThemeMenu themes={themes} />

    <SidebarButtons />
  </LazyFlex>
);

export default Sidebar;
