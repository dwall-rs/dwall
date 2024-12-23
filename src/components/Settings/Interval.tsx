import { LazySlider, LazySpace } from "~/lazy";
import SettingsItem from "./item";
import { useAppContext } from "~/context";
import { createSignal } from "solid-js";
import { writeConfigFile } from "~/commands";
import { translate } from "~/utils/i18n";

const Interval = () => {
  const { config, refetchConfig, translations } = useAppContext();

  const [value, setValue] = createSignal(config()?.interval);

  const onChange = async (v: number) => {
    setValue(v);
    await writeConfigFile({ ...config()!, interval: v });
    refetchConfig();
  };

  return (
    <SettingsItem label={translate(translations()!, "label-check-interval")}>
      <LazySpace gap={8} style={{ color: "var(--colorNeutralForeground1)" }}>
        <LazySlider
          min={15}
          max={300}
          onChange={onChange}
          value={value()}
          style={{ width: "240px" }}
        />
        {value()}
        {translate(translations()!, "unit-second")}
      </LazySpace>
    </SettingsItem>
  );
};

export default Interval;
