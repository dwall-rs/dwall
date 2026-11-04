// Responsibility: Engine run-state text + pulse dot; colors use semantic tokens (success=running, warning=random, muted=stopped).
//       The daemon's last error is rendered in both states: a daemon that stopped for
//       good (no location) leaves that line as the only explanation there is.
import { createMemo, Show } from "solid-js";
import { t } from "@/i18n";
import { engineStore } from "@/store/engine.store";
import { settingsStore } from "@/store/settings.store";
import { useSolarPosition } from "@/hooks/useSolarPosition";
import { resourceValue } from "@/lib/resource";
import { clsx, formatAngle } from "@/utils";
import { openPrivacyLocationSettings } from "@/ipc";
import { toast } from "@/components/ui/toast";

export function EngineStatus() {
  // The readout is labelled "now", so it has to keep up: without polling it froze at
  // whatever the sun was when the window opened (the stage already refreshes its own
  // copy every minute).
  const { position: sp } = useSolarPosition({
    enabled: () => engineStore.running,
    pollMs: 60_000,
  });

  // The mode the engine is actually running comes from the applied config, not the
  // UI toggle — they can differ until the user applies a change.
  const random = createMemo(
    () => settingsStore.config?.wallpaper_mode?.mode === "random",
  );

  // Opened from the status area: the daemon stops instead of retrying when Windows
  // has location switched off for this app, and this is the page that turns it on.
  const openLocationSettings = async () => {
    try {
      await openPrivacyLocationSettings();
    } catch (e) {
      toast.error(t("common.operationFailed"), { description: String(e) });
    }
  };

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
      </Show>

      {/*
        Deliberately outside the running branch. The daemon keeps running after a
        failed cycle (a theme it cannot read), so the status alone would read as "all
        is well"; and it stops for good when the user's own location setting leaves it
        without a position, where the log line it wrote is the only explanation — a
        stopped engine that says only "Engine not running" hides the one fact that
        fixes it.
      */}
      <Show when={engineStore.lastError}>
        {(error) => (
          <div class="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-0.5">
            <p
              data-selectable
              class="max-w-150 truncate text-[11.5px] text-warning"
              title={error()}
            >
              {t("app.engine.lastError")} {error()}
            </p>
            <Show when={!engineStore.running}>
              <button
                type="button"
                class="cursor-pointer text-[11.5px] text-primary hover:underline"
                onClick={() => void openLocationSettings()}
              >
                {t("app.engine.openLocationSettings")}
              </button>
            </Show>
          </div>
        )}
      </Show>
    </div>
  );
}
