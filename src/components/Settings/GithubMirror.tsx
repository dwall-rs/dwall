import { createSignal } from "solid-js";
import { AiFillSave } from "solid-icons/ai";

import { LazyButton, LazyInput } from "~/lazy";
import SettingsItem from "./item";

import { writeConfigFile } from "~/commands";

import { useToast, useConfig, useTranslations } from "~/contexts";

const GithubMirror = () => {
  const { translate } = useTranslations();
  const toast = useToast();
  const { data: config, refetch: refetchConfig } = useConfig();

  const [value, setValue] = createSignal(config()?.github_mirror_template);

  const onChange = (v: string) => {
    setValue(v);
  };

  const onConfirm = async () => {
    await writeConfigFile({ ...config()!, github_mirror_template: value() });
    refetchConfig();
    toast.success(
      translate("message-github-mirror-template-updated", {
        newTemplate: value() ?? "",
      }),
    );
  };

  return (
    <SettingsItem
      layout="vertical"
      label={translate("label-github-mirror-template")}
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
