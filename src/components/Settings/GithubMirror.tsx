import { LazyButton, LazyFlex, LazyInput } from "~/lazy";
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
      <LazyFlex flex={1} justify="round">
        <LazyInput style={{ flex: 15 }} value={value()} onChange={onChange} />
        <LazyFlex flex={1}>
          <LazyButton icon={<AiOutlineCheck />} onClick={onConfirm} />
        </LazyFlex>
      </LazyFlex>
    </SettingsItem>
  );
};

export default GithubMirror;
