import { createMemo, createSignal, Show } from "solid-js";
import { AiOutlineCheck } from "solid-icons/ai";

import { message } from "@tauri-apps/plugin-dialog";

import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
  InputGroupText,
} from "~/components/input-group";
import { Switch } from "~/components/switch";
import { Button } from "~/components/button";
import SettingsItem from "./SettingsItem";

import { writeConfigFile } from "~/commands";

import { useConfig } from "~/contexts";
import { t } from "~/i18n";

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

  const onChange = (v: number) => {
    setValue(v);
    props.onChange(v);
  };

  return (
    <InputGroup>
      <InputGroupInput
        type="number"
        placeholder={props.placeholder}
        min={props.min}
        max={props.max}
        value={value()}
        onChange={onChange}
        autofocus={props.autofocus}
      />
      <InputGroupAddon align="inline-end">
        <InputGroupText>°</InputGroupText>
      </InputGroupAddon>
    </InputGroup>
  );
};

const COORDINATE_LIMITS = {
  LATITUDE: { MIN: -90.0, MAX: 90.0 },
  LONGITUDE: { MIN: -180.0, MAX: 180.0 },
} as const;

const CoordinateSource = () => {
  const { data: config, refetch: refetchConfig } = useConfig();

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
          t("settings.message.switchToManualCoordinatesFailed", {
            error: String(e),
          }),
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
      message(t("settings.message.manualCoordinatesSaved"));
    } catch (e) {
      message(
        t("settings.message.SaveManualCoordinatesFailed", {
          error: String(e),
        }),
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

  return (
    <>
      <SettingsItem
        orientation="horizontal"
        label={t("settings.label.automaticallyRetrieveCoordinates")}
        help={auto() ? undefined : t("settings.help.manuallySetCoordinates")}
      >
        <Switch
          checked={auto()}
          onCheckedChange={handleSwitchCoordinateSource}
        />
      </SettingsItem>

      <Show when={!auto()}>
        <div class="flex items-center justify-end gap-3">
          <CoordinateInput
            placeholder={t("settings.placeholder.longitude")}
            min={COORDINATE_LIMITS.LONGITUDE.MIN}
            max={COORDINATE_LIMITS.LONGITUDE.MAX}
            defaultValue={position().longitude}
            autofocus
            onChange={handlePositionChange("longitude")}
          />
          <CoordinateInput
            placeholder={t("settings.placeholder.latitude")}
            min={COORDINATE_LIMITS.LATITUDE.MIN}
            max={COORDINATE_LIMITS.LATITUDE.MAX}
            defaultValue={position().latitude}
            onChange={handlePositionChange("latitude")}
          />
          <Button
            icon={{ icon: <AiOutlineCheck />, ariaLabel: "Confirm" }}
            onClick={handleConfirmManual}
            disabled={!isPositionValid()}
            variant="outline"
            size="sm"
          />
        </div>
      </Show>
    </>
  );
};

export default CoordinateSource;
