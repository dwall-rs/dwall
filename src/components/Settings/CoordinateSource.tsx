import { LazyButton, LazyInputNumber, LazySpace, LazySwitch } from "~/lazy";
import SettingsItem from "./item";
import { AiOutlineCheck } from "solid-icons/ai";
import { useAppContext } from "~/context";
import { writeConfigFile } from "~/commands";
import { createMemo, createSignal, Show } from "solid-js";

interface CoordinateInputProps {
  min: number;
  max: number;
  placeholder: string;
  defaultValue?: number;
  onChange: (value: number) => void;
}

const CoordinateInput = (props: CoordinateInputProps) => {
  const [value, setValue] = createSignal(props.defaultValue);

  const onChange = (v: number) => {
    setValue(v);
    props.onChange(v);
  };

  return (
    <LazyInputNumber
      placeholder={props.placeholder}
      min={props.min}
      max={props.max}
      value={value()}
      onChange={onChange}
    />
  );
};

const CoordinateSource = () => {
  const { config, refetchConfig } = useAppContext();

  const [auto, setAuto] = createSignal(
    config()?.coordinate_source.type === "AUTOMATIC",
  );

  const [position, setPosition] = createSignal<{
    latitude?: number;
    longitude?: number;
  }>(
    config()?.coordinate_source.type === "AUTOMATIC"
      ? {}
      : {
          latitude: (config()?.coordinate_source as CoordinateSourceManual)
            .latitude,
          longitude: (config()?.coordinate_source as CoordinateSourceManual)
            .longitude,
        },
  );

  const onSwitchCoordinateSource = async () => {
    if (!auto()) {
      await writeConfigFile({
        ...config()!,
        coordinate_source: {
          type: "AUTOMATIC",
        },
      });
      refetchConfig();
    }

    setAuto((prev) => !prev);
  };

  const postionIsValid = createMemo(
    () =>
      position().latitude !== undefined &&
      position().latitude! >= -90.0 &&
      position().latitude! <= 90.0 &&
      position().longitude !== undefined &&
      position().longitude! >= -180.0 &&
      position().longitude! <= 180.0,
  );

  const onConfirmManual = async () => {
    await writeConfigFile({
      ...config()!,
      coordinate_source: {
        type: "MANUAL",
        latitude: position().latitude!,
        longitude: position().longitude!,
      },
    });
    refetchConfig();
  };

  return (
    <SettingsItem label="自动获取坐标">
      <LazySpace gap={auto() ? 0 : 8}>
        <LazySwitch checked={auto()} onChange={onSwitchCoordinateSource} />

        <Show when={!auto()}>
          <LazySpace gap={8}>
            <CoordinateInput
              placeholder="经度"
              min={-180.0}
              max={180.0}
              defaultValue={position().longitude}
              onChange={(v) =>
                setPosition((prev) => ({
                  ...prev,
                  longitude: v,
                }))
              }
            />

            <CoordinateInput
              placeholder="纬度"
              min={-90.0}
              max={90.0}
              defaultValue={position().latitude}
              onChange={(v) =>
                setPosition((prev) => ({
                  ...prev,
                  latitude: v,
                }))
              }
            />

            <LazyButton
              icon={<AiOutlineCheck />}
              onClick={onConfirmManual}
              disabled={!postionIsValid()}
            />
          </LazySpace>
        </Show>
      </LazySpace>
    </SettingsItem>
  );
};

export default CoordinateSource;
