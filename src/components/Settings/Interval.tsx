import { LazyButton, LazySpace } from "~/lazy";
import SettingsItem from "./item";
import { useAppContext } from "~/context";
import { createSignal } from "solid-js";
import { writeConfigFile } from "~/commands";
import { translate } from "~/utils/i18n";
import NumericInput from "../NumericInput";
import { AiFillSave } from "solid-icons/ai";

const Interval = () => {
  const { config, refetchConfig, translations } = useAppContext();

  const [value, setValue] = createSignal(config()?.interval);

  const onSave = async () => {
    await writeConfigFile({ ...config()!, interval: value()! });
    refetchConfig();
  };

  const onChange = async (v?: number) => {
    setValue(v);
  };

  return (
    <SettingsItem label={translate(translations()!, "label-check-interval")}>
      <LazySpace gap={8} style={{ color: "var(--colorNeutralForeground1)" }}>
        <NumericInput
          appearance="underline"
          min={15}
          max={300}
          onChange={onChange}
          value={value()}
          style={{ width: "120px" }}
          contentAfter={
            <LazyButton
              disabled={!value()}
              onClick={onSave}
              icon={<AiFillSave />}
              appearance="subtle"
              size="small"
            />
          }
        />
        {translate(translations()!, "unit-second")}
      </LazySpace>
    </SettingsItem>
  );
};

export default Interval;
