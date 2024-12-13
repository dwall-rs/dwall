import { children } from "solid-js";
import { LazyFlex, LazyTooltip } from "~/lazy";

interface ThemeMenuProps {
  themes: ThemeItem[];
  index?: number;
  appliedThemeID?: string;
  onMenuItemClick: (idx: number) => void;
}

export const ThemeMenu = (props: ThemeMenuProps) => {
  const menu = children(() =>
    props.themes.map((item, idx) => (
      <div
        onClick={() => props.onMenuItemClick(idx)}
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
          <img src={item.thumbnail[0]} alt={item.id} width={64} />
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
