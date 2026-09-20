/* ===== src/components/settings/EngineSection.tsx ===== */
// 职责：引擎与太阳角匹配分组——开机/检查间隔/坐标/锁屏。
import { Switch } from "@/components/ui/switch";
import { patch, save, settingsStore } from "@/store/settings.store";
import { SettingsGroup } from "./SettingsGroup";
import { SettingsRow } from "./SettingsRow";
import { createResource, createSignal } from "solid-js";
import { LoaderCircle, Save, Sun } from "lucide-solid";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
  InputGroupText,
} from "../ui/input-group";
import { checkAutoStart, disableAutoStart, enableAutoStart } from "~/commands";

export function EngineSection() {
  const [flash, setFlash] = createSignal<string | null>(null);
  const ok = async (k: string) => {
    setFlash(k);
    await save();
    setFlash(null);
  };

  const [autoStart, { refetch: refetchAutoStart }] =
    createResource(checkAutoStart);

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
                onChange={async (v) => {
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
                    ok("interval");
                  }}
                  // disabled={
                  //   settingsStore.config?.interval ===
                  //   settingsStore.config?.intervalSaved
                  // }
                />
              </InputGroupAddon>
            </InputGroup>
          }
        />
        <SettingsRow
          label="自动获取坐标"
          okShow={flash() === "coord"}
          desc="经纬度用于计算太阳高度角以匹配壁纸。关闭后需在配置中手动填写坐标。"
          control={
            <Switch
              checked={
                settingsStore.config?.position_source.type === "AUTOMATIC"
              }
              onCheckedChange={(checked) => {
                if (!checked) {
                  // TODO: 开启手动坐标输入框
                }
                // ok("coord");
              }}
            />
          }
        />
        <SettingsRow
          label="同时设置锁屏壁纸"
          okShow={flash() === "lock"}
          desc="若不希望锁屏与桌面同步更换，请关闭此项。"
          control={
            <Switch
              checked={settingsStore.config?.lock_screen_wallpaper_enabled}
              onCheckedChange={(checked) => {
                // toggleCfg("lock");
                patch({ lock_screen_wallpaper_enabled: checked });
                ok("lock");
              }}
            />
          }
        />
      </div>
    </SettingsGroup>
  );
}
