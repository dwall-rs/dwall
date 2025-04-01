import { Show } from "solid-js";
import { LazyFlex } from "~/lazy";
import ImageCarousel from "./ImageCarousel";
import { ThemeActions } from "./ThemeActions";
import Download from "./Download";
import { useTheme } from "~/contexts";

const ThemeShowcase = () => {
  const theme = useTheme();

  return (
    <LazyFlex
      direction="vertical"
      gap={theme.downloadThemeID() ? 8 : 16}
      justify="center"
      align="center"
      style={{ position: "relative" }}
    >
      <ImageCarousel />

      <ThemeActions />

      <Show when={theme.downloadThemeID()}>
        <Download />
      </Show>
    </LazyFlex>
  );
};

export default ThemeShowcase;
