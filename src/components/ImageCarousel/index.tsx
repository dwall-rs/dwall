import { BiSolidChevronLeft, BiSolidChevronRight } from "solid-icons/bi";
import { createSignal, createEffect, onCleanup } from "solid-js";
import { styled } from "solid-styled-components";
import { LazyButton } from "~/lazy";

interface ImageCarouselProps {
  images: Array<{
    src: string;
    alt?: string;
  }>;
  interval?: number; // 切换间隔，单位毫秒
  width?: string;
  height?: string;
}

const Container = styled("div")`
  position: relative;
  overflow: hidden;
  width: 100%;
  height: 100%;
`;

const ImageWrapper = styled("div")`
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  opacity: 0;
  transition: opacity 0.5s ease-in-out;
  display: flex;
  align-items: center;
  justify-content: center;
  &.active {
    opacity: 1;
  }
`;

const Indicators = styled("div")`
  position: absolute;
  bottom: 10px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 8px;
  z-index: 2;
`;

const Indicator = styled("button")`
  width: 10px;
  height: 10px;
  border-radius: 50%;
  border: none;
  background: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  padding: 0;
  &.active {
    background: white;
  }
`;

export default function ImageCarousel(props: ImageCarouselProps) {
  const [currentIndex, setCurrentIndex] = createSignal(0);
  const [isPlaying, setIsPlaying] = createSignal(true);

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
    <Container
      style={{ width: "480px", height: "480px" }}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      {props.images.map((image, index) => (
        <ImageWrapper class={index === currentIndex() ? "active" : ""}>
          <img
            src={image.src}
            alt={image.alt || `Slide ${index + 1}`}
            style={{
              width: props.width || "480px",
              height: props.height || "auto",
              //width: "100%",
              //height: "100%",
              "object-fit": "cover",
            }}
          />
        </ImageWrapper>
      ))}

      <LazyButton
        style={{
          "z-index": 2,
          position: "absolute",
          top: "50%",
          transform: "translateY(-50%)",
        }}
        icon={<BiSolidChevronLeft />}
        shape="circular"
        onClick={prevImage}
      />

      <LazyButton
        style={{
          "z-index": 2,
          position: "absolute",
          top: "50%",
          transform: "translateY(-50%)",
          right: 0,
        }}
        icon={<BiSolidChevronRight />}
        shape="circular"
        onClick={nextImage}
      />

      <Indicators>
        {props.images.map((_, index) => (
          <Indicator
            class={index === currentIndex() ? "active" : ""}
            onClick={() => goToImage(index)}
          />
        ))}
      </Indicators>
    </Container>
  );
}
