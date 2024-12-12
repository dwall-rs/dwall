import { LazyButton, LazyInput } from "~/lazy";
import SettingsItem from "./item";
import { useAppContext } from "~/context";
import { createSignal } from "solid-js";
import { AiFillSave } from "solid-icons/ai";
import { writeConfigFile } from "~/commands";

const GithubMirror = () => {
  const { config, refetchConfig } = useAppContext();

  const [value, setValue] = createSignal(config()?.github_mirror_template);

  const onChange = (v: string) => {
    setValue(v);
  };

  const onConfirm = async () => {
    await writeConfigFile({ ...config()!, github_mirror_template: value() });
    refetchConfig();
  };

  return (
    <SettingsItem label="Github 镜像模板" vertical>
      <LazyInput
        style={{ flex: 15 }}
        value={value()}
        onChange={onChange}
        contentAfter={
          <LazyButton
            appearance="subtle"
            icon={<AiFillSave />}
            onClick={onConfirm}
            size="small"
          />
        }
      />
    </SettingsItem>
  );
};

export default GithubMirror;
