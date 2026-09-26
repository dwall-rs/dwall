/* ===== src/components/settings/UpdateCheck.tsx ===== */
// 职责：更新检查控件——闲置 / 检查中 / 最新 / 可更新 / 下载中 / 待重启 / 失败。
import { Match, Switch } from "solid-js";
import { t } from "@/i18n";
import { check, install, restart, updateStore } from "@/store/update.store";
import { Button } from "@/components/ui/button";
import {
  Check,
  Download,
  LoaderCircle,
  RefreshCw,
  TriangleAlert,
} from "lucide-solid";

export function UpdateCheck() {
  const percent = () =>
    updateStore.progress == null
      ? null
      : Math.round(updateStore.progress * 100);

  return (
    <div class="mt-auto pt-4">
      <Switch>
        <Match when={updateStore.status === "available"}>
          <div class="rounded-xl border border-primary/30 bg-primary/10 p-3">
            <div class="flex items-center gap-2">
              <Download class="size-3.5 shrink-0 text-primary" />
              <span class="font-display text-[12.5px] font-bold text-primary">
                {t("settings.about.available", {
                  version: updateStore.version ?? "",
                })}
              </span>
            </div>
            <Button
              size="sm"
              class="mt-2.5 w-full"
              onClick={() => void install()}
            >
              {t("settings.about.download")}
            </Button>
          </div>
        </Match>

        <Match when={updateStore.status === "downloading"}>
          <div class="rounded-xl border border-border-2 bg-secondary p-3">
            <div class="flex items-center gap-2 font-mono text-[11.5px] text-muted-foreground">
              <LoaderCircle class="size-3.5 shrink-0 animate-spin text-primary" />
              {t("settings.about.downloading")}
              {percent() != null && (
                <span class="ml-auto text-foreground">{percent()}%</span>
              )}
            </div>
            <div
              class="mt-2 h-1 overflow-hidden rounded-full bg-border-2"
              role="progressbar"
              aria-valuenow={percent() ?? 0}
              aria-valuemin={0}
              aria-valuemax={100}
            >
              <div
                class="h-full rounded-full bg-primary transition-[width] duration-200"
                style={{ width: `${percent() ?? 0}%` }}
              />
            </div>
          </div>
        </Match>

        <Match when={updateStore.status === "latest"}>
          <div class="flex items-center gap-2 font-mono text-[11.5px] text-muted-foreground">
            <Check class="size-3.5 shrink-0 text-success" />
            {t("settings.about.upToDate")}
            <Button
              variant="ghost"
              size="xs"
              class="ml-auto"
              icon={<RefreshCw class="size-3.5" />}
              aria-label={t("settings.about.checkUpdate")}
              onClick={() => void check()}
            />
          </div>
        </Match>

        <Match when={updateStore.status === "ready"}>
          <div class="flex items-center gap-2 font-mono text-[11.5px] text-muted-foreground">
            <Check class="size-3.5 shrink-0 text-success" />
            {t("settings.about.ready")}
            <Button
              variant="ghost"
              size="xs"
              class="ml-auto"
              icon={<RefreshCw class="size-3.5" />}
              aria-label={t("settings.about.ready")}
              onClick={() => void restart()}
            />
          </div>
        </Match>

        <Match when={updateStore.status === "error"}>
          <div class="rounded-xl border border-destructive/30 bg-destructive/10 p-3">
            <div class="flex items-center gap-2 font-mono text-[11.5px] text-destructive">
              <TriangleAlert class="size-3.5 shrink-0" />
              {t("settings.about.failed")}
            </div>
            <Button
              variant="outline"
              size="sm"
              class="mt-2.5 w-full"
              onClick={() => void check()}
            >
              {t("settings.about.retry")}
            </Button>
          </div>
        </Match>

        <Match when={true}>
          <Button
            variant="outline"
            size="sm"
            class="w-full"
            disabled={updateStore.status === "checking"}
            icon={
              updateStore.status === "checking" ? (
                <LoaderCircle class="size-3.5 animate-spin" />
              ) : (
                <RefreshCw class="size-3.5" />
              )
            }
            onClick={() => void check()}
          >
            {updateStore.status === "checking"
              ? t("settings.about.checking")
              : t("settings.about.checkUpdate")}
          </Button>
        </Match>
      </Switch>
    </div>
  );
}
