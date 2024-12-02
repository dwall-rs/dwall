import { LazyButton, LazyInput, LazySpaceCompact } from "~/lazy";
import SettingsItem from "./item";
import { useAppContext } from "~/context";
import { createSignal } from "solid-js";
import { AiOutlineCheck } from "solid-icons/ai";
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
      <LazySpaceCompact>
        <LazyInput value={value()} onChange={onChange} />
        <LazyButton icon={<AiOutlineCheck />} onClick={onConfirm} />
      </LazySpaceCompact>
    </SettingsItem>
  );
};

export default GithubMirror;
