import { createEffect, createMemo, createSignal, For } from "solid-js";
import { createStore } from "solid-js/store";

import {
  Carousel,
  CarouselContent,
  CarouselItem,
  CarouselNext,
  CarouselPrevious,
} from "~/components/carousel";
import { AspectRatio } from "~/components/aspect-ratio";
import ThemeImage from "~/components/Image";
import { Label } from "~/components/label";
import { Select } from "~/components/select";

import { generateGitHubThumbnailMirrorUrl } from "~/utils/proxy";

import { useConfig, useMonitor, useTheme, useThemesContext } from "~/contexts";
import { clsx } from "~/utils";
import ThemeActions from "./Actions";
import { t } from "~/i18n";
import { Skeleton } from "~/components/skeleton";

interface ThemeProps {
  id: string;
}

export const Theme = (props: ThemeProps) => {
  const { data: config } = useConfig();
  const { allThemes } = useThemesContext();
  const {
    list: monitors,
    handleChange: handleMonitorChange,
    id: monitorID,
  } = useMonitor();
  const { downloadingTheme } = useTheme();

  let aspectRatioRef!: HTMLDivElement;

  const [indicatorsBottom, setIndicatorsBottom] = createSignal(10);
  const [nameTop, setNameTop] = createSignal(8);

  const [store, setStore] = createStore({ count: 0, current: 0 });

  const handleCarouselChange = (count: number, index: number) => {
    setStore({ count, current: index });
  };

  const theme = createMemo(() => {
    return config() && allThemes().find((t) => t.id === props.id);
  });

  const isCustomized = createMemo(() => {
    const t = theme();
    return t && "author" in t;
  });

  const themeName = createMemo(() => {
    const t = theme();
    return t && "theme_name" in t ? t.theme_name : props.id;
  });

  const imageFormat = createMemo(() => {
    const t = theme();
    return t && "image_format" in t ? t.image_format : "jpeg";
  });

  createEffect(() => {
    setStore({ count: theme()?.thumbnails.length, current: 0 });
  });

  return (
    <div class="flex flex-col items-center justify-between flex-1 pb-4">
      <div class="flex justify-center items-center w-96 gap-3">
        <Label class="flex-1 whitespace-nowrap">
          {t("theme.label.selectMonitor")}
        </Label>
        <Select
          class="flex-4"
          options={monitors()}
          value={monitorID()}
          onChange={handleMonitorChange}
          disabled={downloadingTheme()}
        />
      </div>

      <div class="relative">
        <Carousel
          class="w-full max-w-108"
          onChange={handleCarouselChange}
          id={props.id}
        >
          <CarouselContent>
            <For each={theme()?.thumbnails}>
              {(src, index) => (
                <CarouselItem class="min-w-108">
                  <AspectRatio
                    ref={aspectRatioRef}
                    ratio={1}
                    class="flex items-center justify-center"
                  >
                    <ThemeImage
                      src={
                        config()?.network &&
                        typeof config()?.network === "string"
                          ? generateGitHubThumbnailMirrorUrl(
                              src,
                              config()!.network as string,
                            )
                          : src
                      }
                      isLocal={isCustomized()}
                      class="rounded-md"
                      themeID={props.id}
                      serialNumber={index() + 1}
                      onLoad={({ bottom, top }) => {
                        const aspectRatioBottom =
                          aspectRatioRef!.getBoundingClientRect().bottom;
                        setIndicatorsBottom(aspectRatioBottom - bottom + 4);

                        const aspectRatioTop =
                          aspectRatioRef!.getBoundingClientRect().top;
                        setNameTop(top - aspectRatioTop + 8);
                      }}
                      skeleton={<Skeleton class="absolute w-108 h-108" />}
                    />
                  </AspectRatio>
                </CarouselItem>
              )}
            </For>
          </CarouselContent>

          <CarouselPrevious />
          <CarouselNext />
        </Carousel>

        <p
          class="absolute top-(--name-top) right-2 bg-neutral-950/30 rounded-2xl text-white dark:text-white/60 py-0.5 px-2 text-xs font-medium backdrop-blur-sm"
          style={{ "--name-top": `${nameTop()}px` }}
        >
          {themeName()}
        </p>

        <div
          class="absolute left-1/2 -translate-x-1/2 bottom-(--indicators-bottom) flex flex-col items-center justify-center gap-1"
          style={{ "--indicators-bottom": `${indicatorsBottom()}px` }}
        >
          <div class="flex gap-1.5 z-10 py-1 px-1.5 bg-neutral-950/30 rounded-2xl backdrop-blur-sm">
            <For each={theme()?.thumbnails}>
              {(_, index) => (
                <button
                  type="button"
                  class={clsx(
                    "size-2 rounded-full border-none p-0 bg-white/50 dark:bg-white/40 hover:bg-white/80 dark:hover:bg-white/60 transition-all",
                    store.current === index() &&
                      "bg-white dark:bg-white/60 scale-120",
                  )}
                  aria-label={`Go to slide ${index() + 1}`}
                />
              )}
            </For>
          </div>
        </div>
      </div>

      <div class="flex items-center justify-center w-full relative">
        <ThemeActions
          currentThemeID={props.id}
          themesDirectory={
            isCustomized()
              ? config()?.customized_themes_directory
              : config()?.themes_directory
          }
          isCustomized={isCustomized()}
          imageFormat={imageFormat()}
        />
      </div>
    </div>
  );
};
