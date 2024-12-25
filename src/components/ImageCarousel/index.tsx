import { BiSolidChevronLeft, BiSolidChevronRight } from "solid-icons/bi";
import { createSignal, createEffect, onCleanup } from "solid-js";
import { LazyButton } from "~/lazy";
import "./index.scss";
import Image from "../Image";

interface ImageCarouselProps {
  themeID: string;
  images: Array<{
    src: string;
    alt?: string;
  }>;
  interval?: number;
}

export default function ImageCarousel(props: ImageCarouselProps) {
  const [currentIndex, setCurrentIndex] = createSignal(0);
  const [isPlaying, setIsPlaying] = createSignal(true);
  const [indicatorsBottom, setIndicatorsBottom] = createSignal(10);
  const [isHovered, setIsHovered] = createSignal(false);

  let containerRef: HTMLDivElement | undefined;

  // reset index
  createEffect(() => props.images && setCurrentIndex(0));

  createEffect(() => {
    if (!isPlaying()) return;

    const timer = setInterval(() => {
      nextImage();
    }, props.interval || 3000);

    onCleanup(() => clearInterval(timer));
  });

  const nextImage = () => {
    setCurrentIndex((current) =>
      current === props.images.length - 1 ? 0 : current + 1,
    );
  };

  const prevImage = () => {
    setCurrentIndex((current) =>
      current === 0 ? props.images.length - 1 : current - 1,
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
    <div
      ref={containerRef}
      class="fluent-carousel"
      style={{ width: "480px", height: "480px" }}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      {props.images.map((image, index) => (
        <div
          class={`fluent-carousel-slide ${index === currentIndex() ? "active" : ""
            }`}
        >
          <Image
            src={image.src}
            class="fluent-carousel-image"
            themeID={props.themeID}
            serialNumber={index + 1}
            width={480}
            height={480}
            onLoad={({ width, height }) => {
              const clientHeight = height / (width / 480);
              setIndicatorsBottom((480 - clientHeight) / 2 + 10);
            }}
          />
        </div>
      ))}

      <div class={`fluent-carousel-controls ${isHovered() ? "visible" : ""}`}>
        <LazyButton
          class="fluent-carousel-button prev"
          icon={<BiSolidChevronLeft />}
          shape="circular"
          onClick={prevImage}
        />

        <LazyButton
          class="fluent-carousel-button next"
          icon={<BiSolidChevronRight />}
          shape="circular"
          onClick={nextImage}
        />
      </div>

      <div
        class="fluent-carousel-indicators"
        style={{ bottom: `${indicatorsBottom()}px` }}
      >
        {props.images.map((_, index) => (
          <button
            type="button"
            class={`fluent-indicator ${index === currentIndex() ? "active" : ""}`}
            onClick={() => goToImage(index)}
            aria-label={`Go to slide ${index + 1}`}
          />
        ))}
      </div>
    </div>
  );
}
