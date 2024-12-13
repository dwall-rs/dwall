import { Show } from "solid-js";
import { LazyFlex } from "~/lazy";
import ImageCarousel from "./ImageCarousel";
import { ThemeActions } from "./ThemeActions";
import Download from "./Download";

interface ThemeShowcaseProps {
  currentTheme: () => ThemeItem;
  themeExists: () => boolean;
  appliedThemeID: () => string | undefined;
  downloadThemeID: () => string | undefined;
  setDownloadThemeID: (id?: string) => void;
  onDownload: () => void;
  onApply: () => Promise<void>;
  onCloseTask: () => Promise<void>;
  onMenuItemClick: (index: number) => void;
  index: () => number;
}

const ThemeShowcase = (props: ThemeShowcaseProps) => {
  return (
    <LazyFlex
      direction="vertical"
      gap={16}
      justify="center"
      align="center"
      style={{ position: "relative" }}
    >
      <ImageCarousel
        images={props.currentTheme().thumbnail.map((src) => ({
          src,
          alt: props.currentTheme().id,
        }))}
      />

      <ThemeActions
        themeExists={props.themeExists()}
        appliedThemeID={props.appliedThemeID()}
        currentThemeID={props.currentTheme().id}
        onDownload={() => props.setDownloadThemeID(props.currentTheme().id)}
        onApply={props.onApply}
        onCloseTask={props.onCloseTask}
        downloadThemeID={props.downloadThemeID()}
      />

      <Show when={props.downloadThemeID()}>
        <Download
          themeID={props.downloadThemeID()!}
          onFinished={() => {
            props.setDownloadThemeID();
            props.onMenuItemClick(props.index());
          }}
        />
      </Show>
    </LazyFlex>
  );
};

export default ThemeShowcase;
