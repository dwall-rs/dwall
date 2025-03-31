import { createSignal } from "solid-js";
import { AiFillSave } from "solid-icons/ai";

import { LazyButton, LazySpace } from "~/lazy";
import SettingsItem from "./item";
import NumericInput from "~/components/NumericInput";

import { writeConfigFile } from "~/commands";

import { useConfig, useTranslations } from "~/contexts";

const Interval = () => {
  const { data: config, refetch: refetchConfig } = useConfig();
  const { translate } = useTranslations();

  const [value, setValue] = createSignal(config()?.interval);

  const onSave = async () => {
    await writeConfigFile({ ...config()!, interval: value()! });
    refetchConfig();
  };

  const onChange = async (v?: number) => {
    setValue(v);
  };

  return (
    <SettingsItem label={translate("label-check-interval")}>
      <NumericInput
        appearance="underline"
        min={15}
        max={300}
        onChange={onChange}
        value={value()}
        style={{ width: "100px" }}
        contentAfter={
          <LazySpace gap={4}>
            <span>{translate("unit-second")}</span>
            <LazyButton
              disabled={!value()}
              onClick={onSave}
              icon={<AiFillSave />}
              appearance="subtle"
              size="small"
            />
          </LazySpace>
        }
      />
    </SettingsItem>
  );
};

export default Interval;
