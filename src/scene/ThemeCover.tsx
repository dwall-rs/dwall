/* ===== src/components/scene/ThemeCover.tsx ===== */
// 职责：主题代表缩略——取最接近正午的一张壁纸，cover 填充。所有列表缩略统一走它。
import { createMemo } from "solid-js";
import { Scene } from "./Scene";
import { themeById, themeCover } from "@/domain/themes";

export function ThemeCover(props: { themeId: string; class?: string }) {
  const theme = createMemo(() => themeById(props.themeId));
  const cover = createMemo(() => themeCover(theme()));
  return (
    <Scene
      type={theme().type}
      solar={cover().solar}
      w={cover().w}
      h={cover().h}
      fit="cover"
      class={props.class}
    />
  );
}
