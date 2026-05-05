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

import { useConfig, useMonitor, useTheme } from "~/contexts";
import { type ThemeID, themes } from "~/themes";
import { clsx } from "~/utils";
import ThemeActions from "./Actions";
import { t } from "~/i18n";

interface ThemeProps {
  id: ThemeID;
}

export const Theme = (props: ThemeProps) => {
  const { data: config } = useConfig();
  const {
    list: monitors,
    handleChange: handleMonitorChange,
    id: monitorID,
  } = useMonitor();
  const { downloadingTheme } = useTheme();

  let aspectRatioRef: HTMLDivElement | undefined;

  const [indicatorsBottom, setIndicatorsBottom] = createSignal(10);
  const [nameTop, setNameTop] = createSignal(8);

  const [store, setStore] = createStore({ count: 0, current: 0 });

  const handleCarouselChange = (count: number, index: number) => {
    setStore({ count, current: index });
  };

  const images = createMemo(() => {
    return (config() && themes.find((t) => t.id === props.id)?.thumbnail) ?? [];
  });

  createEffect(() => {
    setStore({ count: images().length, current: 0 });
  });

  return (
    <div class="flex flex-col items-center justify-between flex-1 pb-4">
      <div class="flex justify-center items-center w-96 gap-3">
        <Label class="flex-1">{t("theme.label.selectMonitor")}</Label>
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
            <For each={images()}>
              {(src, index) => (
                <CarouselItem>
                  <AspectRatio
                    ref={aspectRatioRef}
                    ratio={1}
                    class="flex items-center justify-center"
                  >
                    <ThemeImage
                      src={generateGitHubThumbnailMirrorUrl(
                        src,
                        config()!.github_mirror_template,
                      )}
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
          class="absolute top-(--name-top) right-2 bg-neutral-950/30 rounded-2xl text-white dark:text-neutral-950 py-0.5 px-2 text-xs font-medium backdrop-blur-sm"
          style={{ "--name-top": `${nameTop()}px` }}
        >
          {props.id}
        </p>

        <div
          class="absolute left-1/2 -translate-x-1/2 bottom-(--indicators-bottom) flex flex-col items-center justify-center gap-1"
          style={{ "--indicators-bottom": `${indicatorsBottom()}px` }}
        >
          <div class="flex gap-1.5 z-10 py-1 px-1.5 bg-neutral-950/30 rounded-2xl backdrop-blur-sm">
            <For each={images()}>
              {(_, index) => (
                <button
                  type="button"
                  class={clsx(
                    "size-2 rounded-full border-none p-0 bg-white/50 dark:bg-neutral-950/30 hover:bg-white/80 dark:hover:bg-neutral-950/50 transition-all",
                    store.current === index() &&
                      "bg-white dark:bg-neutral-950 scale-120",
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
          themesDirectory={config()?.themes_directory}
        />
      </div>
    </div>
  );
};
