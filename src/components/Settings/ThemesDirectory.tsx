import { LazyButton, LazySpace, LazyTooltip } from "~/lazy";
import SettingsItem from "./item";
import { useAppContext } from "~/context";
import { createSignal } from "solid-js";
import { moveThemesDirectory, openDir } from "~/commands";
import { confirm, message, open } from "@tauri-apps/plugin-dialog";
import { useTranslations } from "~/contexts";

const ThemesDirectory = () => {
  const { config, refetchConfig } = useAppContext();
  const { translate } = useTranslations();

  const [path, setPath] = createSignal(config()?.themes_directory);

  const onOpenThemesDirectory = () => {
    openDir(path()!);
  };

  const onChangePath = async () => {
    const dirPath = await open({ directory: true });
    if (!dirPath) return;

    const newThemesDirectory = `${dirPath}\\themes`;

    const ok = await confirm(
      translate("message-change-themes-directory", {
        newThemesDirectory,
      }),
    );
    if (!ok) return;

    try {
      await moveThemesDirectory(config()!, newThemesDirectory);
      message(
        translate("message-themes-directory-moved", {
          newThemesDirectory,
        }),
      );
      setPath(newThemesDirectory);
      refetchConfig();
    } catch (e) {
      message(String(e), { kind: "error" });
    }
  };

  return (
    <SettingsItem
      layout="vertical"
      label={translate("label-themes-directory")}
      vertical
    >
      <LazySpace gap={8} justify="between">
        <LazyTooltip
          content={translate("tooltip-open-themes-directory")}
          relationship="label"
          withArrow
        >
          <LazyButton
            appearance="transparent"
            style={{ padding: 0 }}
            onClick={onOpenThemesDirectory}
          >
            {path()}
          </LazyButton>
        </LazyTooltip>

        <LazyButton size="small" appearance="primary" onClick={onChangePath}>
          {translate("button-select-folder")}
        </LazyButton>
      </LazySpace>
    </SettingsItem>
  );
};

export default ThemesDirectory;
