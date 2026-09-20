/* ===== src/components/views/SettingsView.tsx ===== */
// 职责：设置视图——标题 + 导语 + 各分组（分组各自单一职责）。
import { AppearanceSection } from "@/components/settings/AppearanceSection";
import { EngineSection } from "@/components/settings/EngineSection";
import { PathsSection } from "@/components/settings/PathsSection";
import { AboutSection } from "@/components/settings/AboutSection";
import { createMemo, Show } from "solid-js";
import {
  discard,
  isConfigDirty,
  save,
  settingsStore,
} from "~/store/settings.store";
import { clsx } from "~/utils";
import { Button } from "~/components/ui/button";
import { Loader2, Save } from "lucide-solid";

export function SettingsView() {
  const dirty = createMemo(() => isConfigDirty(settingsStore));
  return (
    <div class="mx-auto max-w-190 px-7 pb-16 pt-8">
      <div class="mb-1.5 flex items-center gap-3">
        <h2 class="font-display text-[26px] font-extrabold tracking-tight">
          设置
        </h2>
        <span class="font-mono text-[12px] text-muted-foreground">
          preferences
        </span>

        <Show when={settingsStore.status === "ready"}>
          <div class="ml-auto flex items-center gap-2">
            <Show when={settingsStore.saveError}>
              <span class="text-[12px] text-destructive">保存失败</span>
            </Show>

            <span
              class={clsx(
                "flex items-center gap-1.5 text-[12px] text-warning transition-opacity",
                dirty() ? "opacity-100" : "opacity-0",
              )}
            >
              <span class="size-1.5 rounded-full bg-warning animate-bdot" />
              未保存
            </span>
            <Button
              variant="ghost"
              size="sm"
              disabled={!dirty}
              onClick={discard}
            >
              放弃
            </Button>
            <Button
              size="sm"
              disabled={!dirty() || settingsStore.saving}
              onClick={save}
              class={clsx(
                dirty() && !settingsStore.saving && "animate-breathe",
              )}
            >
              <Show
                when={settingsStore.saving}
                fallback={
                  <>
                    <Save class="size-3.5" />
                    保存配置
                  </>
                }
              >
                <Loader2 class="size-3.5 animate-spin" />
                写入中…
              </Show>
            </Button>
          </div>
        </Show>
      </div>
      <p class="mb-7 max-w-145 text-[13px] leading-relaxed text-muted-foreground">
        偏好控制引擎进程与太阳角匹配的行为。开关即时写入配置；带保存图标的字段需手动确认。
      </p>
      <AppearanceSection />
      <EngineSection />
      <PathsSection />
      <AboutSection />
    </div>
  );
}
