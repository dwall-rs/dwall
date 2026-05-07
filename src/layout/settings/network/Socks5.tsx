import { createStore } from "solid-js/store";
import { AiFillSave } from "solid-icons/ai";

import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "~/components/input-group";
import SettingsItem from "../SettingsItem";
import { t } from "~/i18n";
import { useConfig } from "~/contexts";
import { writeConfigFile } from "~/commands";
import { toast } from "~/components/toast";

export default (props: { defaultValue?: Socks5 }) => {
  const { data: config, refetch: refetchConfig } = useConfig();

  const [value, setValue] = createStore<Socks5>(
    props.defaultValue ?? {
      host: "127.0.0.1",
      port: 1080,
    },
  );

  const onSave = async () => {
    try {
      await writeConfigFile({ ...config()!, network: value });
      refetchConfig();
    } catch (e) {
      toast.error(
        t("settings.message.socks5UpdateFailed", { error: String(e) }),
      );
    }
  };

  return (
    <SettingsItem
      label={t("settings.label.socks5")}
      help={t("settings.help.socks5")}
    >
      <InputGroup class="max-w-54">
        <InputGroupInput
          value={value.host}
          placeholder="host"
          class="flex-3"
          onChange={(v) => setValue("host", v)}
        />
        :
        <InputGroupInput
          type="number"
          value={value.port}
          placeholder="port"
          class="flex-2"
          onChange={(v) => setValue("port", v)}
        />
        <InputGroupAddon align="inline-end">
          <InputGroupButton
            variant="ghost"
            size="xs"
            onClick={onSave}
            icon={<AiFillSave />}
          />
        </InputGroupAddon>
      </InputGroup>
    </SettingsItem>
  );
};
