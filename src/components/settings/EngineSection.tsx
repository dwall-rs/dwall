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
      title="引擎与太阳角匹配"
      delay={80}
    >
      <div class="overflow-hidden rounded-[15px] border border-border bg-card shadow-[0_1px_2px_rgba(0,0,0,.05),0_8px_24px_rgba(0,0,0,.06)]">
        <SettingsRow
          label="开机启动"
          okShow={flash() === "startup"}
          desc="仅启动后台引擎进程，不打开图形界面，几乎不占内存。"
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
          label="检查间隔"
          okShow={flash() === "interval"}
          desc={
            <>
              引擎每隔 <span class="text-(--warning)">N 秒</span>
              重算一次太阳高度角，命中对应时段的壁纸。固定模式无固定切换时间，全靠此轮询驱动。
            </>
          }
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
                <InputGroupText>s</InputGroupText>
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
          label="自动获取坐标"
          okShow={flash() === "coord"}
          desc="经纬度用于计算太阳高度角以匹配壁纸。关闭后可手动填写坐标。"
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
            label="手动坐标"
            okShow={flash() === "manual"}
            desc="纬度 -90~90、经度 -180~180、海拔（米）。"
            control={
              <div class="flex items-end gap-3">
                <FieldGroup class="grid flex-1 grid-cols-3 gap-3">
                  <Field>
                    <FieldLabel>纬度</FieldLabel>
                    <Input
                      type="number"
                      value={manual().latitude}
                      onChange={(v) => patchPosition({ latitude: v })}
                    />
                  </Field>
                  <Field>
                    <FieldLabel>经度</FieldLabel>
                    <Input
                      type="number"
                      value={manual().longitude}
                      onChange={(v) => patchPosition({ longitude: v })}
                    />
                  </Field>
                  <Field>
                    <FieldLabel>海拔</FieldLabel>
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
                  保存
                </Button>
              </div>
            }
          />
        </Show>
        <SettingsRow
          label="同时设置锁屏壁纸"
          okShow={flash() === "lock"}
          desc="若不希望锁屏与桌面同步更换，请关闭此项。"
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
