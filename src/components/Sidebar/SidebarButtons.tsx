import { Show } from "solid-js";
import { AiFillSetting } from "solid-icons/ai";
import { TbArrowBigUpLinesFilled } from "solid-icons/tb";
import { LazyButton, LazySpace, LazyTooltip } from "~/lazy";
import { useSettings, useTheme, useTranslations, useUpdate } from "~/contexts";
import styles from "./index.module.scss";

const SidebarButtons = () => {
  const { translate } = useTranslations();
  const { setMenuItemIndex, downloadThemeID } = useTheme();
  const { update: updateIsAvailable, setShowUpdateDialog } = useUpdate();
  const { setShowSettings } = useSettings();

  const onUpdate = () => {
    updateIsAvailable() && setShowUpdateDialog(true);
  };

  return (
    <LazySpace
      direction="column"
      gap="s"
      justify="end"
      class={styles.sidebarButtons}
    >
      <Show when={updateIsAvailable()}>
        <LazyTooltip
          positioning="after"
          content={translate("tooltip-new-version-available")}
          relationship="label"
          withArrow
        >
          <LazyButton
            appearance="transparent"
            shape="circular"
            icon={<TbArrowBigUpLinesFilled />}
            onClick={onUpdate}
            disabled={!!downloadThemeID()} // 下载主题时禁用更新按钮
          />
        </LazyTooltip>
      </Show>

      <LazyTooltip
        positioning="after"
        content={translate("tooltip-settings")}
        relationship="label"
        withArrow
      >
        <LazyButton
          appearance="transparent"
          shape="circular"
          icon={<AiFillSetting />}
          onClick={() => {
            setShowSettings(true);
            setMenuItemIndex();
          }}
          disabled={!!downloadThemeID()} // 下载主题时禁用设置按钮
        />
      </LazyTooltip>
    </LazySpace>
  );
};

export default SidebarButtons;
