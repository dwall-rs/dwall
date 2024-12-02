import { LazyInputNumber, LazySpace } from "~/lazy";
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
        <LazyInputNumber
          min={1}
          max={600}
          value={value()}
          onChange={onChange}
        />
        秒
      </LazySpace>
    </SettingsItem>
  );
};

export default Interval;
