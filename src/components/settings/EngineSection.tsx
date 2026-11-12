// Responsibility: Engine & solar-angle matching group — launch at startup / check interval / coordinates (automatic or manual) / lock screen.
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import {
  patch,
  patchAutomatic,
  patchPosition,
  positionCacheMode,
  type PositionCacheMode,
  save,
  setAutoDetectColorScheme,
  setPositionCacheMode,
  setPositionType,
  settingsStore,
} from "@/store/settings.store";
import { SettingsGroup } from "./SettingsGroup";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { SettingsRow } from "./SettingsRow";
import { t } from "@/i18n";
import {
  createMemo,
  createResource,
  createSignal,
  onCleanup,
  onMount,
  Show,
} from "solid-js";
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
import {
  checkAutoStart,
  disableAutoStart,
  enableAutoStart,
  pickImage,
} from "@/ipc";
import { resourceValue } from "@/lib/resource";
import { colorSchemePauseStore } from "@/store/color-scheme-pause.store";
import { toast } from "@/components/ui/toast";

export function EngineSection() {
  const [flash, setFlash] = createSignal<string | null>(null);
  // The countdown has to move without a backend round trip, and the pause can also
  // start while the window is open (the daemon notices a manual theme change), so the
  // state is polled rather than assumed.
  const [tick, setTick] = createSignal(0);
  onMount(() => {
    void colorSchemePauseStore.refreshColorSchemePause();
    const timer = setInterval(() => {
      void colorSchemePauseStore.refreshColorSchemePause();
      setTick((value) => value + 1);
    }, 30_000);
    onCleanup(() => clearInterval(timer));
  });
  const pauseRemaining = createMemo(() => {
    tick();
    return colorSchemePauseStore.remainingMinutes();
  });
  const ok = async (k: string) => {
    setFlash(k);
    await save();
    setFlash(null);
  };

  const chooseBatteryWallpaper = async () => {
    const path = await pickImage();
    if (!path) return;

    patch({ battery_wallpaper: path });
    void ok("batteryWallpaper");
  };

  const [autoStart, { refetch: refetchAutoStart }] =
    createResource(checkAutoStart);

  const manual = createMemo(() => {
    const ps = settingsStore.config?.position_source;
    return ps?.type === "MANUAL"
      ? ps
      : { type: "MANUAL" as const, latitude: 0, longitude: 0, altitude: 875 };
  });

  const automatic = createMemo(() => {
    const ps = settingsStore.config?.position_source;
    return ps?.type === "AUTOMATIC"
      ? ps
      : { type: "AUTOMATIC" as const, cache_minutes: 120 };
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
              checked={resourceValue(autoStart) ?? false}
              onCheckedChange={async (checked) => {
                try {
                  if (checked) {
                    await enableAutoStart();
                  } else {
                    await disableAutoStart();
                  }
                } catch (e) {
                  toast.error(t("common.operationFailed"), {
                    description: String(e),
                  });
                }
                // Re-read either way: on failure the switch falls back to the real state.
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
                max={3600}
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
                // Switching to manual only reveals the coordinate fields. Saving here
                // wrote 0,0 — the placeholder seed — into config.toml, and a running
                // daemon applied it until the user finished typing; the row's own save
                // button is what commits coordinates.
                setPositionType(checked ? "AUTOMATIC" : "MANUAL");
                if (checked) void ok("coord");
              }}
            />
          }
        />
        <Show when={settingsStore.config?.position_source.type === "AUTOMATIC"}>
          <SettingsRow
            label={t("settings.engine.positionCache")}
            okShow={flash() === "positionCache"}
            desc={t("settings.engine.positionCacheDesc")}
            control={
              <div class="flex items-center gap-2">
                <Select
                  value={positionCacheMode()}
                  onValueChange={(value) => {
                    setPositionCacheMode(value as PositionCacheMode);
                    void ok("positionCache");
                  }}
                >
                  <SelectTrigger class="min-w-38" variant="outline">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="every-calculation">
                      {t("settings.engine.cacheEveryCalculation")}
                    </SelectItem>
                    <SelectItem value="interval">
                      {t("settings.engine.cacheInterval")}
                    </SelectItem>
                    <SelectItem value="startup-only">
                      {t("settings.engine.cacheStartupOnly")}
                    </SelectItem>
                  </SelectContent>
                </Select>
                <Show when={positionCacheMode() === "interval"}>
                  <InputGroup class="w-28">
                    <InputGroupInput
                      type="number"
                      min={1}
                      max={1440}
                      value={automatic().cache_minutes}
                      onChange={(v) =>
                        // The field is a number input, so "1.5" arrives as a float and
                        // the daemon reads the key as an integer: rounding here is what
                        // keeps a fractional value out of the configuration file.
                        patchAutomatic({
                          cache_minutes: Math.min(
                            1440,
                            Math.max(1, Math.round(Number(v))),
                          ),
                        })
                      }
                    />
                    <InputGroupAddon align="inline-end">
                      <InputGroupText>
                        {t("settings.engine.minutesUnit")}
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
                          void ok("positionCache");
                        }}
                      />
                    </InputGroupAddon>
                  </InputGroup>
                </Show>
              </div>
            }
          />
        </Show>
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
                      min={-90}
                      max={90}
                      step={0.0001}
                      value={manual().latitude}
                      onChange={(v) => patchPosition({ latitude: v })}
                    />
                  </Field>
                  <Field>
                    <FieldLabel>{t("settings.engine.longitude")}</FieldLabel>
                    <Input
                      type="number"
                      min={-180}
                      max={180}
                      step={0.0001}
                      value={manual().longitude}
                      onChange={(v) => patchPosition({ longitude: v })}
                    />
                  </Field>
                  <Field>
                    <FieldLabel>{t("settings.engine.altitude")}</FieldLabel>
                    <Input
                      type="number"
                      min={-500}
                      max={9000}
                      step={1}
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
        <SettingsRow
          label={t("settings.engine.colorScheme")}
          okShow={flash() === "scheme"}
          desc={t("settings.engine.colorSchemeDesc")}
          control={
            <Switch
              checked={settingsStore.config?.auto_detect_color_scheme}
              onCheckedChange={(checked) => {
                setAutoDetectColorScheme(checked);
                void ok("scheme");
              }}
            />
          }
        />
        <Show when={settingsStore.config?.auto_detect_color_scheme}>
          <SettingsRow
            label={t("settings.engine.colorSchemePause")}
            desc={
              <Show
                when={colorSchemePauseStore.pause()}
                fallback={t("settings.engine.colorSchemePauseDesc")}
              >
                {t("settings.engine.pauseRemaining", {
                  minutes: pauseRemaining(),
                })}{" "}
                <Show
                  when={colorSchemePauseStore.pause()?.cause === "external"}
                >
                  {t("settings.engine.pauseDetected")}
                </Show>
              </Show>
            }
            control={
              <Show
                when={colorSchemePauseStore.pause()}
                fallback={
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() =>
                      void colorSchemePauseStore.setColorSchemePauseFor(60)
                    }
                  >
                    {t("settings.engine.pauseHour")}
                  </Button>
                }
              >
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() =>
                    void colorSchemePauseStore.setColorSchemePauseFor(0)
                  }
                >
                  {t("settings.engine.resume")}
                </Button>
              </Show>
            }
          />
        </Show>
        <div class="border-b border-border px-[18px] py-4">
          <div class="text-[14.5px] font-bold!">
            {t("settings.engine.onBattery")}
          </div>
          <div class="mt-1 max-w-110 text-[12px] leading-relaxed text-muted-foreground">
            {t("settings.engine.onBatteryDesc")}
          </div>
        </div>
        <SettingsRow
          label={t("settings.engine.darkOnBattery")}
          okShow={flash() === "batteryDark"}
          desc={t("settings.engine.darkOnBatteryDesc")}
          control={
            <Switch
              checked={settingsStore.config?.dark_on_battery}
              disabled={
                !settingsStore.config?.auto_detect_color_scheme ||
                settingsStore.saving
              }
              onCheckedChange={(checked) => {
                patch({ dark_on_battery: checked });
                void ok("batteryDark");
              }}
            />
          }
        />
        <SettingsRow
          label={t("settings.engine.batteryWallpaper")}
          okShow={flash() === "batteryWallpaper"}
          desc={
            <>
              {t("settings.engine.batteryWallpaperDesc")}
              <Show when={settingsStore.config?.battery_wallpaper}>
                <span
                  data-selectable
                  class="mt-1 block truncate font-mono text-[11px] text-foreground/70"
                >
                  {settingsStore.config?.battery_wallpaper}
                </span>
              </Show>
            </>
          }
          control={
            <div class="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                disabled={settingsStore.saving}
                onClick={() => void chooseBatteryWallpaper()}
              >
                {t("settings.engine.batteryWallpaperChoose")}
              </Button>
              <Switch
                checked={!!settingsStore.config?.battery_wallpaper}
                disabled={settingsStore.saving}
                onCheckedChange={(checked) => {
                  if (checked) {
                    void chooseBatteryWallpaper();
                  } else {
                    patch({ battery_wallpaper: null });
                    void ok("batteryWallpaper");
                  }
                }}
              />
            </div>
          }
        />
      </div>
    </SettingsGroup>
  );
}
