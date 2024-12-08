import { LazySlider, LazySpace } from "~/lazy";
import SettingsItem from "./item";
import { useAppContext } from "~/context";
import { createSignal } from "solid-js";
import { writeConfigFile } from "~/commands";

const Interval = () => {
  const { config, refetchConfig } = useAppContext();

  const [value, setValue] = createSignal(config()?.interval);

  const onChange = async (v: number) => {
    setValue(v);
    await writeConfigFile({ ...config()!, interval: v });
    refetchConfig();
  };

  return (
    <SettingsItem label="检测间隔">
      <LazySpace gap={8}>
        <LazySlider
          min={15}
          max={300}
          onChange={onChange}
          value={value()}
          style={{ width: "240px" }}
        />
        {value()}秒
      </LazySpace>
    </SettingsItem>
  );
};

export default Interval;
