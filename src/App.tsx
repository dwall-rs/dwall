import { children, createSignal, onMount } from "solid-js";
import { showMainWindow } from "./commands";
import { LazyButton, LazyFlex, LazySpace, LazyText } from "./lazy";
import Catalina1 from "~/assets/thumbnail/Catalina/1.avif";
import Catalina2 from "~/assets/thumbnail/Catalina/2.avif";
import Catalina3 from "~/assets/thumbnail/Catalina/3.avif";
import Catalina4 from "~/assets/thumbnail/Catalina/4.avif";
import Catalina5 from "~/assets/thumbnail/Catalina/5.avif";
import Catalina6 from "~/assets/thumbnail/Catalina/6.avif";
import Catalina7 from "~/assets/thumbnail/Catalina/7.avif";
import Catalina8 from "~/assets/thumbnail/Catalina/8.avif";
import BigSur1 from "~/assets/thumbnail/BigSur/1.avif";
import BigSur2 from "~/assets/thumbnail/BigSur/2.avif";
import BigSur3 from "~/assets/thumbnail/BigSur/3.avif";
import BigSur4 from "~/assets/thumbnail/BigSur/4.avif";
import BigSur5 from "~/assets/thumbnail/BigSur/5.avif";
import BigSur6 from "~/assets/thumbnail/BigSur/6.avif";
import BigSur7 from "~/assets/thumbnail/BigSur/7.avif";
import BigSur8 from "~/assets/thumbnail/BigSur/8.avif";
import ImageCarousel from "./components/ImageCarousel";
import "./App.scss";

const images = [
  {
    id: "Catalina",
    thumbnail: [
      Catalina1,
      Catalina2,
      Catalina3,
      Catalina4,
      Catalina5,
      Catalina6,
      Catalina7,
      Catalina8,
    ],
  },
  {
    id: "Big Sur",
    thumbnail: [
      BigSur1,
      BigSur2,
      BigSur3,
      BigSur4,
      BigSur5,
      BigSur6,
      BigSur7,
      BigSur8,
    ],
  },
];

const App = () => {
  const [index, setIndex] = createSignal(0);

  onMount(() => {
    showMainWindow();
  });

  const menu = children(() =>
    images.map((item, idx) => (
      <div
        onClick={() => setIndex(idx)}
        classList={{ "menu-item": true, active: idx === index() }}
      >
        <LazyFlex direction="vertical" justify="center" align="center">
          <img src={item.thumbnail[0]} alt="Catalina" width={64} />
          <LazyText strong>{item.id}</LazyText>
        </LazyFlex>
      </div>
    )),
  );

  return (
    <LazyFlex
      style={{ height: "100vh" }}
      gap={8}
      justify="center"
      align="center"
    >
      <LazyFlex direction="vertical" gap={8} class="menu">
        {menu()}
      </LazyFlex>

      <LazyFlex direction="vertical" gap={8} justify="center" align="center">
        <ImageCarousel
          images={images[index()].thumbnail.map((s) => ({
            src: s,
            alt: images[index()].id,
          }))}
          height="480px"
          width="480px"
        />

        <LazySpace gap={8}>
          <LazyButton type="primary">下载</LazyButton>
          <LazyButton type="primary">应用</LazyButton>
        </LazySpace>
      </LazyFlex>
    </LazyFlex>
  );
};

export default App;
