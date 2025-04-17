import { createSignal } from "solid-js";
import { AiFillSave } from "solid-icons/ai";

import { open } from "@tauri-apps/plugin-shell";

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
      help={{
        content: translate("help-github-mirror-template"),
        onClick: () =>
          open(
            "https://gh-proxy.com/gist.githubusercontent.com/thep0y/682ebeb2b8d4f6eea3841fe3f42c0e30/raw/2f5b641e77abe3cb8f74ee8f65ead95beb663444/markdown",
          ),
      }}
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
