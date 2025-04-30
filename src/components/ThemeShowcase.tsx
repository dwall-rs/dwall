import { Show } from "solid-js";

import { LazyFlex } from "~/lazy";
import Download from "./Download";
import ImageCarousel from "./ImageCarousel";
import { ThemeActions } from "./ThemeActions";

import { useTheme } from "~/contexts";

const ThemeShowcase = () => {
  const theme = useTheme();

  return (
    <LazyFlex
      direction="column"
      gap="l"
      justify="center"
      align="center"
      style={{ position: "relative" }}
    >
      <ImageCarousel />

      <div style={{ height: "32px" }}>
        <Show when={theme.downloadThemeID()} fallback={<ThemeActions />}>
          <Download />
        </Show>
      </div>
    </LazyFlex>
  );
};

export default ThemeShowcase;
