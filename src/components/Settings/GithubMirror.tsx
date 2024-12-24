import { LazyButton, LazyInput } from "~/lazy";
import SettingsItem from "./item";
import { useAppContext } from "~/context";
import { createSignal } from "solid-js";
import { AiFillSave } from "solid-icons/ai";
import { writeConfigFile } from "~/commands";
import { translate } from "~/utils/i18n";

const GithubMirror = () => {
  const { config, refetchConfig, translations } = useAppContext();

  const [value, setValue] = createSignal(config()?.github_mirror_template);

  const onChange = (v: string) => {
    setValue(v);
  };

  const onConfirm = async () => {
    await writeConfigFile({ ...config()!, github_mirror_template: value() });
    refetchConfig();
  };

  return (
    <SettingsItem
      layout="vertical"
      label={translate(translations()!, "label-github-mirror-template")}
      vertical
    >
      <LazyInput
        style={{ flex: 15 }}
        appearance="filled-lighter"
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
