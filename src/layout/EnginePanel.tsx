// Responsibility: the engine's live state and its one action — start, or stop after a
// confirmation. Lives in the sidebar so the answer to "is it running?" is never a page away.
import { createMemo, createSignal, Show } from "solid-js";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { t } from "@/i18n";
import { engineStore, stop, toggle } from "@/store/engine.store";
import { modeStore } from "@/store/mode.store";
import { settingsStore } from "@/store/settings.store";
import { useSolarPosition } from "@/hooks/useSolarPosition";
import { resourceValue } from "@/lib/resource";
import { clsx, formatAngle } from "@/utils";
import { openPrivacyLocationSettings } from "@/ipc";
import { toast } from "@/components/ui/toast";
import {
  LoaderCircle,
  MapPin,
  Power,
  Square,
  TriangleAlert,
} from "lucide-solid";

export function EnginePanel() {
  const [confirmOpen, setConfirmOpen] = createSignal(false);
  const [busy, setBusy] = createSignal(false);

  const { position } = useSolarPosition({
    enabled: () => engineStore.running,
    pollMs: 60_000,
  });
  const p = () => resourceValue(position);

  // The applied mode is what the daemon is doing; the UI toggle may already be on the
  // other mode but has not been written yet.
  const mode = createMemo(() => {
    const applied = settingsStore.config?.wallpaper_mode?.mode;
    return applied === "random"
      ? "random"
      : applied === "fixed"
        ? "fixed"
        : modeStore.mode;
  });

  const start = async () => {
    if (busy() || engineStore.running) return;
    setBusy(true);
    try {
      await toggle();
    } finally {
      setBusy(false);
    }
  };

  const stopEngine = async () => {
    setBusy(true);
    try {
      await stop();
      setConfirmOpen(false);
    } finally {
      setBusy(false);
    }
  };

  const openLocationSettings = async () => {
    try {
      await openPrivacyLocationSettings();
    } catch (e) {
      toast.error(t("common.operationFailed"), { description: String(e) });
    }
  };

  return (
    <section class="px-3">
      <div class="rounded-[14px] border border-sidebar-border bg-card/70 p-3">
        <div class="flex items-center gap-2">
          <span
            class={clsx(
              "size-2 shrink-0 rounded-full",
              engineStore.running
                ? mode() === "random"
                  ? "bg-warning animate-pulse-ring text-warning"
                  : "bg-success animate-pulse-ring text-success"
                : "bg-muted-foreground",
            )}
          />
          <span class="min-w-0 flex-1 truncate text-[12.5px] font-medium">
            {engineStore.running
              ? t("app.engine.runningShort")
              : t("app.engine.stoppedShort")}
          </span>
          <span class="rounded-full bg-muted px-2 py-0.5 font-mono text-[10px] text-muted-foreground">
            {t(mode() === "random" ? "app.mode.random" : "app.mode.fixed")}
          </span>
        </div>

        <Show when={engineStore.running}>
          <div class="mt-2.5 flex items-center gap-2 rounded-lg bg-muted/55 px-2.5 py-1.5 font-mono text-[11px] text-muted-foreground">
            <Show when={p()} fallback={<span>—</span>}>
              {(pos) => (
                <>
                  <span
                    class={
                      pos().altitude < 0
                        ? "text-muted-foreground"
                        : "text-solar"
                    }
                  >
                    {pos().altitude < 0 ? "☾" : "☀"}
                  </span>
                  <span>
                    {t("app.engine.altitude")}{" "}
                    <span class="text-foreground">
                      {formatAngle(pos().altitude)}°
                    </span>
                  </span>
                  <span class="text-border-2">|</span>
                  <span>
                    {t("app.engine.azimuth")}{" "}
                    <span class="text-foreground">
                      {formatAngle(pos().azimuth)}°
                    </span>
                  </span>
                </>
              )}
            </Show>
          </div>
        </Show>

        <Show when={engineStore.lastError}>
          {(error) => (
            <div class="mt-2.5 flex items-start gap-1.5 text-[11px] leading-relaxed text-warning">
              <TriangleAlert class="mt-0.5 size-3 shrink-0" />
              <p data-selectable class="line-clamp-3 min-w-0" title={error()}>
                {error()}
              </p>
            </div>
          )}
        </Show>

        <Show when={engineStore.lastError && !engineStore.running}>
          <button
            type="button"
            onClick={() => void openLocationSettings()}
            class="mt-2 inline-flex cursor-pointer items-center gap-1 text-[11px] font-medium text-primary hover:underline"
          >
            <MapPin class="size-3" />
            {t("app.engine.openLocationSettings")}
          </button>
        </Show>

        <Button
          size="sm"
          variant={engineStore.running ? "outline" : "primary"}
          disabled={busy()}
          onClick={() =>
            engineStore.running ? setConfirmOpen(true) : void start()
          }
          class="mt-3 w-full justify-center"
        >
          <Show
            when={busy()}
            fallback={
              engineStore.running ? (
                <Square class="size-3 fill-current" />
              ) : (
                <Power class="size-3.5" />
              )
            }
          >
            <LoaderCircle class="size-3.5 animate-spin" />
          </Show>
          {engineStore.running ? t("app.engine.stop") : t("app.engine.start")}
        </Button>
      </div>

      <Dialog open={confirmOpen()} onOpenChange={setConfirmOpen}>
        <DialogContent class="max-w-105">
          <DialogHeader>
            <DialogTitle>{t("app.engine.confirmTitle")}</DialogTitle>
            <DialogDescription>{t("app.engine.confirmBody")}</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="ghost"
              onClick={() => setConfirmOpen(false)}
              disabled={busy()}
            >
              {t("common.cancel")}
            </Button>
            <Button
              variant="destructive"
              onClick={() => void stopEngine()}
              disabled={busy()}
            >
              <Show when={busy()} fallback={<Square class="size-3" />}>
                <LoaderCircle class="size-3 animate-spin" />
              </Show>
              {t("app.engine.confirmAction")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </section>
  );
}
