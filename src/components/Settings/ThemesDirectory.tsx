import { LazyButton, LazySpace, LazyTooltip } from "~/lazy";
import SettingsItem from "./item";
import { useAppContext } from "~/context";
import { createSignal } from "solid-js";
import { moveThemesDirectory, openDir } from "~/commands";
import { confirm, message, open } from "@tauri-apps/plugin-dialog";
import { translate } from "~/utils/i18n";

const ThemesDirectory = () => {
  const { config, refetchConfig, translations } = useAppContext();

  const [path, setPath] = createSignal(config()?.themes_directory);

  const onOpenThemesDirectory = () => {
    openDir(path()!);
  };

  const onChangePath = async () => {
    const dirPath = await open({ directory: true });
    if (!dirPath) return;

    const newThemesDirectory = `${dirPath}\\themes`;

    const ok = await confirm(
      translate(translations()!, "message-change-themes-directory", {
        newThemesDirectory,
      }),
    );
    if (!ok) return;

    try {
      await moveThemesDirectory(config()!, newThemesDirectory);
      message(
        translate(translations()!, "message-themes-directory-moved", {
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
      label={translate(translations()!, "label-themes-directory")}
      vertical
    >
      <LazySpace gap={8} justify="between">
        <LazyTooltip
          content={translate(translations()!, "tooltip-open-themes-directory")}
          relationship="label"
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
          {translate(translations()!, "button-select-folder")}
        </LazyButton>
      </LazySpace>
    </SettingsItem>
  );
};

export default ThemesDirectory;
