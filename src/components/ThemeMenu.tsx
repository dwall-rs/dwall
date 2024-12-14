import { children } from "solid-js";
import { LazyFlex, LazyTooltip } from "~/lazy";
import Image from "./Image";

interface ThemeMenuProps {
  themes: ThemeItem[];
  index?: number;
  appliedThemeID?: string;
  onMenuItemClick: (idx: number, height: number) => void;
}

export const ThemeMenu = (props: ThemeMenuProps) => {
  const heights: Record<string, number> = {};

  const menu = children(() =>
    props.themes.map((item, idx) => (
      <div
        onClick={() => props.onMenuItemClick(idx, heights[item.id])}
        classList={{
          "menu-item": true,
          active: idx === props.index,
          applied: item.id === props.appliedThemeID,
        }}
      >
        <LazyTooltip
          placement="right"
          text={
            props.appliedThemeID === item.id
              ? `${item.id}（正在使用）`
              : item.id
          }
          delay={500}
          showArrow
        >
          <Image
            src={item.thumbnail[0]}
            width={64}
            height={64}
            onLoad={(height) => {
              heights[item.id] = height;
            }}
          />
        </LazyTooltip>
      </div>
    )),
  );

  return (
    <LazyFlex direction="vertical" gap={8} class="menu">
      {menu()}
    </LazyFlex>
  );
};
