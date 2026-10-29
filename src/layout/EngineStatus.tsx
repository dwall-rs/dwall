// Responsibility: Engine run-state text + pulse dot; colors use semantic tokens (success=running, warning=random, muted=stopped).
import { createMemo, Show } from "solid-js";
import { t } from "@/i18n";
import { engineStore } from "@/store/engine.store";
import { settingsStore } from "@/store/settings.store";
import { useSolarPosition } from "@/hooks/useSolarPosition";
import { resourceValue } from "@/lib/resource";
import { clsx, formatAngle } from "@/utils";

export function EngineStatus() {
  const { position: sp } = useSolarPosition({
    enabled: () => engineStore.running,
  });

  // The mode the engine is actually running comes from the applied config, not the
  // UI toggle — they can differ until the user applies a change.
  const random = createMemo(
    () => settingsStore.config?.wallpaper_mode?.mode === "random",
  );

  return (
    <div class="flex min-w-0 flex-col gap-1">
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
              random() ? "bg-warning text-warning" : "bg-success text-success",
            )}
          />
          {t("app.engine.running", {
            mode: t(random() ? "app.mode.random" : "app.mode.fixed"),
          })}
          <span class="inline-flex items-center gap-1.5 font-display font-semibold text-foreground">
            <Show when={resourceValue(sp)} fallback="—">
              {(p) => (
                <>
                  <span>{p().altitude < 0 ? "☾" : "☀"}</span>
                  <span>
                    {t("app.engine.altitude")} {formatAngle(p().altitude)}°
                  </span>
                  <span class="text-muted-foreground">·</span>
                  <span>
                    {t("app.engine.azimuth")} {formatAngle(p().azimuth)}°
                  </span>
                </>
              )}
            </Show>
          </span>
        </div>

        {/*
          The daemon keeps running after a failed cycle (a denied location
          permission, a theme it cannot read), so the status alone would read as
          "all is well"; the log line it wrote is the only explanation there is.
        */}
        <Show when={engineStore.lastError}>
          {(error) => (
            <p
              data-selectable
              class="max-w-150 truncate text-[11.5px] text-warning"
              title={error()}
            >
              {t("app.engine.lastError")} {error()}
            </p>
          )}
        </Show>
      </Show>
    </div>
  );
}
