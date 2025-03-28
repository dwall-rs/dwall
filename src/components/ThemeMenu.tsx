import { children, createMemo } from "solid-js";
import { LazyBadge, LazyFlex, LazyTooltip } from "~/lazy";
import Image from "./Image";
import { BsCheckLg } from "solid-icons/bs";
import { useAppContext } from "~/context";
import { generateGitHubThumbnailMirrorUrl } from "~/utils/proxy";

interface ThemeMenuProps {
  themes: ThemeItem[];
}

export const ThemeMenu = (props: ThemeMenuProps) => {
  const { theme, settings, config } = useAppContext();
  const heights: Record<string, number> = {};

  const disabled = createMemo(() => !!theme.downloadThemeID());

  const menu = children(() =>
    props.themes.map((item, idx) => (
      <div
        onClick={() => {
          if (disabled()) return; // 下载主题时不允许切换主题

          settings.setShow(false);
          theme.handleThemeSelection(idx);
        }}
        classList={{
          "menu-item": true,
          "menu-item-active": idx === theme.menuItemIndex(),
          "menu-item-applied": item.id === theme.appliedThemeID(),
          "menu-item-disabled": disabled(),
        }}
      >
        <LazyTooltip positioning="after" content={item.id} relationship="label">
          <Image
            src={generateGitHubThumbnailMirrorUrl(
              item.thumbnail[0],
              config()?.github_mirror_template,
            )}
            width={64}
            themeID={item.id}
            serialNumber={1}
            height={64}
            onLoad={({ height }) => {
              heights[item.id] = height;
            }}
          />
        </LazyTooltip>
        {item.id === theme.appliedThemeID() && (
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
    <LazyFlex direction="vertical" gap={8} class="thumbnails-container">
      {menu()}
    </LazyFlex>
  );
};
