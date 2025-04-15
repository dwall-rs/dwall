import { children, createMemo } from "solid-js";
import { LazyBadge, LazyFlex, LazyTooltip } from "~/lazy";
import Image from "../Image";
import { BsCheckLg } from "solid-icons/bs";
import { generateGitHubThumbnailMirrorUrl } from "~/utils/proxy";
import { useConfig, useSettings, useTheme } from "~/contexts";
import styles from "./index.module.scss";

interface ThemeMenuProps {
  themes: ThemeItem[];
}

const ThemeMenu = (props: ThemeMenuProps) => {
  const theme = useTheme();
  const settings = useSettings();
  const { data: config } = useConfig();
  const heights: Record<string, number> = {};

  const disabled = createMemo(() => !!theme.downloadThemeID());

  const menu = children(() =>
    props.themes.map((item, idx) => (
      <div
        onClick={() => {
          if (disabled()) return; // 下载主题时不允许切换主题

          theme.handleThemeSelection(idx);
          settings.setShowSettings(false);
        }}
        classList={{
          [styles.menuItem]: true,
          [styles.menuItemActive]: idx === theme.menuItemIndex(),
          [styles.menuItemApplied]: item.id === theme.appliedThemeID(),
          [styles.menuItemDisabled]: disabled(),
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
          <div class={styles.menuItemAppliedBadge}>
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
    <LazyFlex
      direction="column"
      gap="s"
      class={styles.thumbnailsContainer}
      grow={7}
      shrink={1}
      padding="10px 10px 10px 24px"
    >
      {menu()}
    </LazyFlex>
  );
};

export default ThemeMenu;
