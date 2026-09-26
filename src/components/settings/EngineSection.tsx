/* ===== src/components/settings/EngineSection.tsx ===== */
// 职责：引擎与太阳角匹配分组——开机/检查间隔/坐标（自动或手动）/锁屏。
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import {
  patch,
  patchPosition,
  save,
  setPositionType,
  settingsStore,
} from "@/store/settings.store";
import { SettingsGroup } from "./SettingsGroup";
import { SettingsRow } from "./SettingsRow";
import { t } from "@/i18n";
import { createMemo, createResource, createSignal, Show } from "solid-js";
import { LoaderCircle, Save, Sun } from "lucide-solid";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
  InputGroupText,
} from "../ui/input-group";
import { Field, FieldGroup, FieldLabel } from "../ui/field";
import { Input } from "../ui/input";
import { checkAutoStart, disableAutoStart, enableAutoStart } from "@/ipc";

export function EngineSection() {
  const [flash, setFlash] = createSignal<string | null>(null);
  const ok = async (k: string) => {
    setFlash(k);
    await save();
    setFlash(null);
  };

  const [autoStart, { refetch: refetchAutoStart }] =
    createResource(checkAutoStart);

  const manual = createMemo(() => {
    const ps = settingsStore.config?.position_source;
    return ps?.type === "MANUAL"
      ? ps
      : { type: "MANUAL" as const, latitude: 0, longitude: 0, altitude: 875 };
  });

  return (
    <SettingsGroup
      icon={<Sun class="size-3.25" />}
      title={t("settings.engine.title")}
      delay={80}
    >
      <div class="overflow-hidden rounded-[15px] border border-border bg-card shadow-[0_1px_2px_rgba(0,0,0,.05),0_8px_24px_rgba(0,0,0,.06)]">
        <SettingsRow
          label={t("settings.engine.launchAtStartup")}
          okShow={flash() === "startup"}
          desc={t("settings.engine.launchAtStartupDesc")}
          control={
            <Switch
              checked={autoStart()}
              onCheckedChange={async (checked) => {
                if (checked) {
                  await enableAutoStart();
                } else {
                  await disableAutoStart();
                }
                refetchAutoStart();
              }}
            />
          }
        />
        <SettingsRow
          label={t("settings.engine.interval")}
          okShow={flash() === "interval"}
          desc={t("settings.engine.intervalDesc")}
          control={
            <InputGroup class="w-32">
              <InputGroupInput
                type="number"
                min={1}
                value={settingsStore.config?.interval}
                onChange={(v) => {
                  patch({ interval: v || 15 });
                }}
              />
              <InputGroupAddon align="inline-end">
                <InputGroupText>
                  {t("settings.engine.secondsUnit")}
                </InputGroupText>
              </InputGroupAddon>
              <InputGroupAddon align="inline-end">
                <InputGroupButton
                  icon={
                    settingsStore.saving ? (
                      <LoaderCircle class="size-3.75 animate-spin" />
                    ) : (
                      <Save class="size-3.75" />
                    )
                  }
                  onClick={() => {
                    void ok("interval");
                  }}
                />
              </InputGroupAddon>
            </InputGroup>
          }
        />
        <SettingsRow
          label={t("settings.engine.autoCoords")}
          okShow={flash() === "coord"}
          desc={t("settings.engine.autoCoordsDesc")}
          control={
            <Switch
              checked={
                settingsStore.config?.position_source.type === "AUTOMATIC"
              }
              onCheckedChange={(checked) => {
                setPositionType(checked ? "AUTOMATIC" : "MANUAL");
                void ok("coord");
              }}
            />
          }
        />
        <Show when={settingsStore.config?.position_source.type === "MANUAL"}>
          <SettingsRow
            stacked
            label={t("settings.engine.manualCoords")}
            okShow={flash() === "manual"}
            desc={t("settings.engine.manualCoordsDesc")}
            control={
              <div class="flex items-end gap-3">
                <FieldGroup class="grid flex-1 grid-cols-3 gap-3">
                  <Field>
                    <FieldLabel>{t("settings.engine.latitude")}</FieldLabel>
                    <Input
                      type="number"
                      value={manual().latitude}
                      onChange={(v) => patchPosition({ latitude: v })}
                    />
                  </Field>
                  <Field>
                    <FieldLabel>{t("settings.engine.longitude")}</FieldLabel>
                    <Input
                      type="number"
                      value={manual().longitude}
                      onChange={(v) => patchPosition({ longitude: v })}
                    />
                  </Field>
                  <Field>
                    <FieldLabel>{t("settings.engine.altitude")}</FieldLabel>
                    <Input
                      type="number"
                      value={manual().altitude}
                      onChange={(v) => patchPosition({ altitude: v })}
                    />
                  </Field>
                </FieldGroup>
                <Button
                  size="sm"
                  disabled={settingsStore.saving}
                  onClick={() => void ok("manual")}
                >
                  <Save class="size-3.5" />
                  {t("settings.engine.save")}
                </Button>
              </div>
            }
          />
        </Show>
        <SettingsRow
          label={t("settings.engine.lockScreen")}
          okShow={flash() === "lock"}
          desc={t("settings.engine.lockScreenDesc")}
          control={
            <Switch
              checked={settingsStore.config?.lock_screen_wallpaper_enabled}
              onCheckedChange={(checked) => {
                patch({ lock_screen_wallpaper_enabled: checked });
                void ok("lock");
              }}
            />
          }
        />
      </div>
    </SettingsGroup>
  );
}
