import { themes } from "~/themes";
import { LazyFlex } from "~/lazy";
import ThemeMenu from "~/components/ThemeMenu";
import SidebarButtons from "./SidebarButtons";
import { sidebar } from "./Sidebar.css";

const Sidebar = () => (
  <LazyFlex direction="column" align="center" justify="between" class={sidebar}>
    <ThemeMenu themes={themes} />

    <SidebarButtons />
  </LazyFlex>
);

export default Sidebar;
