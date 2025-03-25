import { LazyButton, LazySpace } from "~/lazy";
import SettingsItem from "./item";
import { useAppContext } from "~/context";
import { createSignal } from "solid-js";
import { writeConfigFile } from "~/commands";
import NumericInput from "../NumericInput";
import { AiFillSave } from "solid-icons/ai";
import { useTranslations } from "../TranslationsContext";

const Interval = () => {
  const { config, refetchConfig } = useAppContext();
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
