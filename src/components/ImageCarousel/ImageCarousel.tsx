import {
  createSignal,
  createEffect,
  onCleanup,
  createMemo,
  For,
} from "solid-js";
import { BiSolidChevronLeft, BiSolidChevronRight } from "solid-icons/bi";

import { useConfig, useTheme } from "~/contexts";

import { LazyButton } from "~/lazy";

import ThemeImage from "~/components/Image";

import { generateGitHubThumbnailMirrorUrl } from "~/utils/proxy";

import * as styles from "./ImageCarousel.css";

interface ImageCarouselProps {
  interval?: number;
}

export default function ImageCarousel(props: ImageCarouselProps) {
  const theme = useTheme();
  const { data: config } = useConfig();

  const [currentIndex, setCurrentIndex] = createSignal(0);
  const [isPlaying, setIsPlaying] = createSignal(true);
  const [indicatorsBottom, setIndicatorsBottom] = createSignal(10);
  const [isHovered, setIsHovered] = createSignal(false);
  const [wrapperHeight, setWrapperHeight] = createSignal("auto");

  const images = createMemo(() => {
    const currentConfig = config();
    if (!currentConfig) return [];

    return theme.currentTheme()!.thumbnail.map((src) => ({
      src: generateGitHubThumbnailMirrorUrl(
        src,
        currentConfig.github_mirror_template,
      ),
      alt: theme.currentTheme()!.id,
    }));
  });

  // reset index
  createEffect(() => images() && setCurrentIndex(0));

  createEffect(() => {
    if (!isPlaying()) return;

    const timer = setInterval(() => {
      nextImage();
    }, props.interval || 3000);

    onCleanup(() => clearInterval(timer));
  });

  const nextImage = () => {
    setCurrentIndex((current) =>
      current === images().length - 1 ? 0 : current + 1,
    );
  };

  const prevImage = () => {
    setCurrentIndex((current) =>
      current === 0 ? images().length - 1 : current - 1,
    );
  };

  const goToImage = (index: number) => {
    setCurrentIndex(index);
  };

  const handleMouseEnter = () => {
    setIsPlaying(false);
    setIsHovered(true);
  };

  const handleMouseLeave = () => {
    setIsPlaying(true);
    setIsHovered(false);
  };

  return (
    <div class={styles.carousel}>
      <div
        class={styles.wrapper}
        style={{ height: wrapperHeight() }}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
      >
        <For each={images()}>
          {(image, index) => (
            <div
              class={`${styles.slide} ${index() === currentIndex() ? styles.activeSlide : ""}`}
            >
              <ThemeImage
                src={image.src}
                class={styles.image}
                themeID={theme.currentTheme()!.id}
                serialNumber={index() + 1}
                onLoad={({ width, height }) => {
                  const clientHeight = height / (width / 480);
                  setWrapperHeight(`${clientHeight}px`);
                  setIndicatorsBottom(10);
                }}
              />
            </div>
          )}
        </For>

        <div
          class={`${styles.controls} ${isHovered() ? styles.visibleControls : ""}`}
        >
          <LazyButton
            class={`${styles.button} ${styles.prevButton}`}
            icon={<BiSolidChevronLeft />}
            shape="circular"
            onClick={prevImage}
          />

          <LazyButton
            class={`${styles.button} ${styles.nextButton}`}
            icon={<BiSolidChevronRight />}
            shape="circular"
            onClick={nextImage}
          />
        </div>

        <div
          class={styles.indicators}
          style={{ bottom: `${indicatorsBottom()}px` }}
        >
          <For each={images()}>
            {(_, index) => (
              <button
                type="button"
                class={`${styles.indicator} ${index() === currentIndex() ? styles.activeIndicator : ""}`}
                onClick={() => goToImage(index())}
                aria-label={`Go to slide ${index() + 1}`}
              />
            )}
          </For>
        </div>
      </div>
    </div>
  );
}
