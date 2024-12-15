import { LazyButton, LazySpace, LazyTooltip } from "~/lazy";
import SettingsItem from "./item";
import { useAppContext } from "~/context";
import { createSignal } from "solid-js";
import { moveThemesDirectory, openDir } from "~/commands";
import { confirm, message, open } from "@tauri-apps/plugin-dialog";

const ThemesDirectory = () => {
  const { config, refetchConfig } = useAppContext();

  const [path, setPath] = createSignal(config()?.themes_directory);

  const onOpenThemesDirectory = () => {
    openDir(path()!);
  };

  const onChangePath = async () => {
    const dirPath = await open({ directory: true });
    if (!dirPath) return;

    const newThemesDirectory = `${dirPath}\\themes`;
    const ok = await confirm(`将主题目录变为：${newThemesDirectory}？`);
    if (!ok) return;

    try {
      await moveThemesDirectory(config()!, newThemesDirectory);
      message(`主题目录已移动至：${newThemesDirectory}`);
      setPath(newThemesDirectory);
      refetchConfig();
    } catch (e) {
      message(String(e), { kind: "error" });
    }
  };

  return (
    <SettingsItem label="主题目录" vertical>
      <LazySpace gap={8} justify="between">
        <LazyTooltip text="单击可打开主题目录" placement="top" delay={1000}>
          <LazyButton
            appearance="transparent"
            style={{ padding: 0 }}
            onClick={onOpenThemesDirectory}
          >
            {path()}
          </LazyButton>
        </LazyTooltip>

        <LazyButton size="small" appearance="primary" onClick={onChangePath}>
          修改
        </LazyButton>
      </LazySpace>
    </SettingsItem>
  );
};

export default ThemesDirectory;
