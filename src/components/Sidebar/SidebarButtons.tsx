import { Show } from "solid-js";
import { AiFillSetting } from "solid-icons/ai";
import { TbArrowBigUpLinesFilled } from "solid-icons/tb";
import { LazyButton, LazySpace, LazyTooltip } from "~/lazy";
import { useSettings, useTheme, useTranslations, useUpdate } from "~/contexts";
import { sidebarButtons, upgradeButton } from "./SidebarButtons.css";

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
      paddingBottom="m"
      paddingRight="10px"
      class={sidebarButtons}
    >
      <Show when={updateIsAvailable()}>
        <LazyTooltip
          positioning="after"
          content={translate("tooltip-new-version-available")}
          relationship="label"
          withArrow
        >
          <LazyButton
            class={upgradeButton}
            appearance="transparent"
            shape="circular"
            icon={<TbArrowBigUpLinesFilled />}
            onClick={onUpdate}
            disabled={!!downloadThemeID()} // Disable update button when downloading theme
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
          disabled={!!downloadThemeID()} // Disable settings button when downloading theme
        />
      </LazyTooltip>
    </LazySpace>
  );
};

export default SidebarButtons;
