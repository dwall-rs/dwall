import { createSignal } from "solid-js";
import { AiFillSave } from "solid-icons/ai";

import { open } from "@tauri-apps/plugin-shell";

import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "~/components/input-group";
import SettingsItem from "./SettingsItem";

import { writeConfigFile } from "~/commands";

import { toast } from "~/components/toast";

import { useConfig } from "~/contexts";

import { Button } from "~/components/button";
import { Compass } from "lucide-solid";
import { t } from "~/i18n";

const GithubMirror = () => {
  const { data: config, refetch: refetchConfig } = useConfig();

  const [value, setValue] = createSignal(config()?.github_mirror_template);

  const handleInput = (v: string) => {
    setValue(v);
  };

  const onConfirm = async () => {
    const mirrorTemplate = value();
    await writeConfigFile({
      ...config()!,
      github_mirror_template: mirrorTemplate,
    });
    refetchConfig();
    toast.success(
      <>
        {t("settings.message.githubMirrorTemplateUpdated")}
        {mirrorTemplate && (
          <code class="font-mono rounded-md bg-neutral-100 py-0.5 px-1 inline-block border border-neutral-200 dark:border-white/10">
            {mirrorTemplate}
          </code>
        )}
      </>,
    );
  };

  return (
    <SettingsItem
      orientation="vertical"
      label={t("settings.label.githubMirrorTemplate")}
      help={
        <span>
          {t("settings.help.githubMirror")}
          <Button
            size="xs"
            variant="outline"
            onClick={() =>
              open(
                "https://gh-proxy.com/gist.githubusercontent.com/thep0y/682ebeb2b8d4f6eea3841fe3f42c0e30/raw/2f5b641e77abe3cb8f74ee8f65ead95beb663444/markdown",
              )
            }
            icon={{ icon: <Compass />, ariaLabel: "Save" }}
          />
        </span>
      }
    >
      {/*<LazyInput
        style={{ flex: 15 }}
        appearance="filled-lighter"
        value={value()}
        onInput={handleInput}
        contentAfter={
          <LazyButton
            appearance="subtle"
            icon={<AiFillSave />}
            onClick={onConfirm}
            size="small"
          />
        }
      />*/}
      <InputGroup>
        <InputGroupInput
          style={{ flex: 15 }}
          value={value()}
          onInput={handleInput}
        />
        <InputGroupAddon align="inline-end">
          <InputGroupButton
            variant="ghost"
            icon={<AiFillSave />}
            onClick={onConfirm}
            size="xs"
          />
        </InputGroupAddon>
      </InputGroup>
    </SettingsItem>
  );
};

export default GithubMirror;
