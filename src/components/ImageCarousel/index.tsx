import { BiSolidChevronLeft, BiSolidChevronRight } from "solid-icons/bi";
import { createSignal, createEffect, onCleanup, onMount } from "solid-js";
import { LazyButton } from "~/lazy";
import "./index.scss";

interface ImageCarouselProps {
  images: Array<{
    src: string;
    alt?: string;
  }>;
  interval?: number; // 切换间隔，单位毫秒
}

export default function ImageCarousel(props: ImageCarouselProps) {
  const [currentIndex, setCurrentIndex] = createSignal(0);
  const [isPlaying, setIsPlaying] = createSignal(true);
  const [imageHeight, setImageHeight] = createSignal(480);

  let containerRef: HTMLDivElement | undefined;

  const updateImageHeight = () => {
    const img = containerRef!.querySelector(".carousel-image");
    setImageHeight(img!.clientHeight);
  };

  onMount(() => {
    const observer = new MutationObserver(updateImageHeight);

    observer.observe(containerRef!, { childList: true });

    onCleanup(() => observer.disconnect());
  });

  // 自动播放
  createEffect(() => {
    if (!isPlaying()) return;

    const timer = setInterval(() => {
      nextImage();
    }, props.interval || 3000);

    onCleanup(() => clearInterval(timer));
  });

  // 切换到下一张图片
  const nextImage = () => {
    setCurrentIndex((current) =>
      current === props.images.length - 1 ? 0 : current + 1,
    );
  };

  // 切换到上一张图片
  const prevImage = () => {
    setCurrentIndex((current) =>
      current === 0 ? props.images.length - 1 : current - 1,
    );
  };

  // 直接跳转到指定图片
  const goToImage = (index: number) => {
    setCurrentIndex(index);
  };

  // 鼠标进入时暂停自动播放
  const handleMouseEnter = () => {
    setIsPlaying(false);
  };

  // 鼠标离开时恢复自动播放
  const handleMouseLeave = () => {
    setIsPlaying(true);
  };

  return (
    <div
      ref={containerRef}
      class="image-carousel-container"
      style={{ width: "480px", height: "480px" }}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      {props.images.map((image, index) => (
        <div
          class={`image-wrapper ${index === currentIndex() ? "active" : ""}`}
        >
          <img
            src={image.src}
            alt={image.alt || `Slide ${index + 1}`}
            class="carousel-image"
          />
        </div>
      ))}

      <LazyButton
        class="prev-button"
        icon={<BiSolidChevronLeft />}
        shape="circular"
        onClick={prevImage}
      />

      <LazyButton
        class="next-button"
        icon={<BiSolidChevronRight />}
        shape="circular"
        onClick={nextImage}
      />

      <div
        class="indicators"
        style={{ bottom: `${(480 - imageHeight()) / 2 + 10}px` }}
      >
        {props.images.map((_, index) => (
          <button
            type="button"
            class={`indicator ${index === currentIndex() ? "active" : ""}`}
            onClick={() => goToImage(index)}
          />
        ))}
      </div>
    </div>
  );
}
