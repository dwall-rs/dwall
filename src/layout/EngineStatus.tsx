// 职责：引擎运行状态文案 + 脉冲点；颜色用语义色（success=运行，warning=随机，muted=停止）。
import { createResource, Show } from "solid-js";
import { currentSolarPosition } from "@/ipc";
import { t } from "@/i18n";
import { engineStore } from "@/store/engine.store";
import { settingsStore } from "@/store/settings.store";
import { uiStore } from "@/store/ui.store";
import { clsx, formatAngle } from "@/utils";

export function EngineStatus() {
  const [sp] = createResource(
    () => settingsStore.config?.position_source,
    (ps) => currentSolarPosition(ps),
  );

  return (
    <Show
      when={engineStore.running}
      fallback={
        <div class="flex items-center gap-2 text-[12.5px] text-muted-foreground">
          <span class="size-2 rounded-full bg-muted-foreground" />
          {t("app.engine.notRunning")}
        </div>
      }
    >
      <div class="flex items-center gap-2 text-[12.5px] text-muted-foreground">
        <span
          class={clsx(
            "size-2 rounded-full text-current animate-pulse-ring",
            uiStore.mode === "random"
              ? "bg-warning text-warning"
              : "bg-success text-success",
          )}
        />
        <Show when={uiStore.mode === "fixed"} fallback={t("app.engine.daily")}>
          {t("app.engine.running")}{" "}
          <span class="inline-flex items-center gap-1 font-display font-semibold text-foreground">
            <Show when={sp()} fallback="—">
              {(p) => (
                <>
                  {p().altitude < 0 ? "☾" : "☀"} {t("app.engine.altitude")}
                  {formatAngle(p().altitude)}° · {t("app.engine.azimuth")}
                  {formatAngle(p().azimuth)}°
                </>
              )}
            </Show>
          </span>
        </Show>
      </div>
    </Show>
  );
}
