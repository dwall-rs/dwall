import { children } from "solid-js";
import { LazyBadge, LazyFlex, LazyTooltip } from "~/lazy";
import Image from "./Image";
import { BsCheckLg } from "solid-icons/bs";

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
          "menu-item-active": idx === props.index,
          "menu-item-applied": item.id === props.appliedThemeID,
        }}
      >
        <LazyTooltip positioning="after" content={item.id} relationship="label">
          <Image
            src={item.thumbnail[0]}
            width={64}
            themeID={item.id}
            serialNumber={1}
            height={64}
            onLoad={({ height }) => {
              heights[item.id] = height;
            }}
          />
        </LazyTooltip>
        {item.id === props.appliedThemeID && (
          <div class="menu-item-applied-badge">
            <LazyBadge
              shape="rounded"
              icon={<BsCheckLg />}
              color="success"
              size="small"
            />
          </div>
        )}
      </div>
    )),
  );

  return (
    <LazyFlex direction="vertical" gap={8} class="menu">
      {menu()}
    </LazyFlex>
  );
};
