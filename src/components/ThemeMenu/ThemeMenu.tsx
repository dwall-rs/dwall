import { children, createMemo } from "solid-js";
import { LazyBadge, LazyFlex, LazyTooltip } from "~/lazy";
import ThemeImage from "../Image";
import { BsCheckLg } from "solid-icons/bs";
import { generateGitHubThumbnailMirrorUrl } from "~/utils/proxy";
import { useConfig, useSettings, useTheme } from "~/contexts";
import * as styles from "./ThemeMenu.css";

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
        role="tab"
        tabIndex={0}
        onClick={() => {
          if (disabled()) return; // Prevent theme switching while downloading

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
          <ThemeImage
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
          <LazyBadge
            class={styles.menuItemAppliedBadge}
            shape="rounded"
            icon={<BsCheckLg />}
            color="success"
            size="small"
          />
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
