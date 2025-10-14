import { children, createMemo, createSignal, Show } from "solid-js";
import { AiOutlineCheck } from "solid-icons/ai";

import { message } from "@tauri-apps/plugin-dialog";

import { LazyButton, LazySpace, LazySwitch } from "~/lazy";
import SettingsItem from "./Item";

import NumericInput from "~/components/NumericInput";

import { writeConfigFile } from "~/commands";

import { useConfig, useTranslations } from "~/contexts";

interface CoordinateInputProps {
  min: number;
  max: number;
  placeholder: string;
  defaultValue?: number;
  onChange: (value?: number) => void;
  autofocus?: boolean;
}

const CoordinateInput = (props: CoordinateInputProps) => {
  const [value, setValue] = createSignal(props.defaultValue);

  const onChange = (v?: number) => {
    setValue(v);
    props.onChange(v);
  };

  return (
    <NumericInput
      placeholder={props.placeholder}
      min={props.min}
      max={props.max}
      value={value()}
      onChange={onChange}
      size="small"
      contentAfter="°"
      appearance="underline"
      autofocus={props.autofocus}
    />
  );
};

const COORDINATE_LIMITS = {
  LATITUDE: { MIN: -90.0, MAX: 90.0 },
  LONGITUDE: { MIN: -180.0, MAX: 180.0 },
} as const;

const CoordinateSource = () => {
  const { data: config, refetch: refetchConfig } = useConfig();
  const { translate } = useTranslations();

  const initialPosition: Omit<PositionSourceManual, "type"> =
    config()?.position_source.type === "MANUAL"
      ? {
          latitude: (config()?.position_source as PositionSourceManual)
            .latitude,
          longitude: (config()?.position_source as PositionSourceManual)
            .longitude,
        }
      : {};

  const [auto, setAuto] = createSignal(
    config()?.position_source.type === "AUTOMATIC",
  );
  const [position, setPosition] = createSignal<{
    latitude?: number;
    longitude?: number;
  }>(initialPosition);

  const isPositionValid = createMemo(() => {
    const { latitude, longitude } = position();
    return (
      latitude !== undefined &&
      longitude !== undefined &&
      latitude >= COORDINATE_LIMITS.LATITUDE.MIN &&
      latitude <= COORDINATE_LIMITS.LATITUDE.MAX &&
      longitude >= COORDINATE_LIMITS.LONGITUDE.MIN &&
      longitude <= COORDINATE_LIMITS.LONGITUDE.MAX
    );
  });

  const handleSwitchCoordinateSource = async () => {
    if (!auto()) {
      try {
        await writeConfigFile({
          ...config()!,
          position_source: {
            type: "AUTOMATIC",
          },
        });
        refetchConfig();
      } catch (e) {
        message(
          translate("message-switching-to-manual-coordinate-config", {
            error: String(e),
          }) as string,
          { kind: "error" },
        );
        return;
      }
    }

    setAuto((prev) => !prev);
  };

  const handleConfirmManual = async () => {
    const { latitude, longitude } = position();
    if (!isPositionValid()) return;

    const newConfig: PositionSourceManual = {
      type: "MANUAL",
      latitude,
      longitude,
    };

    try {
      await writeConfigFile({
        ...config()!,
        position_source: newConfig,
      });
      refetchConfig();
      message(translate("message-coordinates-saved") as string);
    } catch (e) {
      message(
        translate("message-saving-manual-coordinates", {
          error: String(e),
        }) as string,
        {
          kind: "error",
        },
      );
      return;
    }
  };

  const handlePositionChange =
    (field: keyof Omit<PositionSourceManual, "type">) => (value?: number) => {
      setPosition((prev) => ({
        ...prev,
        [field]: value,
      }));
    };

  const latitudePlaceholder = translate("placeholder-latitude");
  const longitudePlaceholder = translate("placeholder-longitude");

  const renderCoordinateInputs = children(() => (
    <Show when={!auto()}>
      <LazySpace gap="l" justify="end">
        <CoordinateInput
          placeholder={longitudePlaceholder}
          min={COORDINATE_LIMITS.LONGITUDE.MIN}
          max={COORDINATE_LIMITS.LONGITUDE.MAX}
          defaultValue={position().longitude}
          autofocus
          onChange={handlePositionChange("longitude")}
        />
        <CoordinateInput
          placeholder={latitudePlaceholder}
          min={COORDINATE_LIMITS.LATITUDE.MIN}
          max={COORDINATE_LIMITS.LATITUDE.MAX}
          defaultValue={position().latitude}
          onChange={handlePositionChange("latitude")}
        />
        <LazyButton
          icon={<AiOutlineCheck />}
          onClick={handleConfirmManual}
          disabled={!isPositionValid()}
          size="small"
        />
      </LazySpace>
    </Show>
  ));

  return (
    <SettingsItem
      layout="horizontal"
      label={translate("label-automatically-retrieve-coordinates")}
      help={auto() ? undefined : translate("help-manually-set-coordinates")}
      extra={renderCoordinateInputs()}
    >
      <LazySwitch checked={auto()} onChange={handleSwitchCoordinateSource} />
    </SettingsItem>
  );
};

export default CoordinateSource;
