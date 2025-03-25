import { AiFillSetting } from "solid-icons/ai";
import { TbArrowBigUpLinesFilled } from "solid-icons/tb";
import { Show } from "solid-js";
import { LazyButton, LazySpace, LazyTooltip } from "~/lazy";
import { useTranslations } from "./TranslationsContext";
import { useAppContext } from "~/context";

const SidebarButtons = () => {
  const { translate } = useTranslations();
  const {
    update: { resource: updateIsAvailable, setShowDialog: setShowUpdateDialog },
    settings: { setShow: setShowSettings },
    theme: { setMenuItemIndex },
  } = useAppContext();

  const onUpdate = () => {
    updateIsAvailable() && setShowUpdateDialog(true);
  };

  return (
    <LazySpace
      direction="vertical"
      gap={8}
      justify="end"
      align="center"
      class="sidebar-buttons"
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
            setMenuItemIndex();
            setShowSettings(true);
          }}
        />
      </LazyTooltip>
    </LazySpace>
  );
};

export default SidebarButtons;
