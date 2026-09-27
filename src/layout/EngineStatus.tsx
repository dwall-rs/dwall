// Responsibility: Engine run-state text + pulse dot; colors use semantic tokens (success=running, warning=random, muted=stopped).
import { Show } from "solid-js";
import { t } from "@/i18n";
import { engineStore } from "@/store/engine.store";
import { modeStore } from "@/store/mode.store";
import { useSolarPosition } from "@/hooks/useSolarPosition";
import { clsx, formatAngle } from "@/utils";

export function EngineStatus() {
  const { position: sp } = useSolarPosition({
    enabled: () => engineStore.running,
  });

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
            modeStore.mode === "random"
              ? "bg-warning text-warning"
              : "bg-success text-success",
          )}
        />
        <Show
          when={modeStore.mode === "fixed"}
          fallback={t("app.engine.daily")}
        >
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
