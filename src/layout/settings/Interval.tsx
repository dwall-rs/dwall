import { createSignal } from "solid-js";
import { AiFillSave } from "solid-icons/ai";

import SettingsItem from "./SettingsItem";

import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
  InputGroupText,
} from "~/components/input-group";

import { writeConfigFile } from "~/commands";

import { useConfig } from "~/contexts";
import { toast } from "~/components/toast";
import { t } from "~/i18n";

const Interval = () => {
  const { data: config, refetch: refetchConfig } = useConfig();

  const [value, setValue] = createSignal(config()?.interval);

  const onSave = async () => {
    try {
      await writeConfigFile({ ...config()!, interval: value()! });
      refetchConfig();
      toast.success(
        t("settings.message.checkIntervalUpdated", {
          interval: value()!.toString(),
        }),
      );
    } catch (e) {
      toast.error(
        t("settings.message.checkIntervalUpdateFailed", { error: String(e) }),
      );
    }
  };

  const onChange = async (v?: number) => {
    setValue(v);
  };

  return (
    <SettingsItem label={t("settings.label.checkInterval")}>
      <InputGroup class="max-w-32">
        <InputGroupInput
          type="number"
          min={15}
          max={300}
          onChange={onChange}
          value={value()}
        />
        <InputGroupAddon align="inline-end">
          <InputGroupText>s</InputGroupText>
          <InputGroupButton
            variant="ghost"
            size="xs"
            onClick={onSave}
            icon={<AiFillSave />}
            disabled={!value()}
          />
        </InputGroupAddon>
      </InputGroup>
    </SettingsItem>
  );
};

export default Interval;
