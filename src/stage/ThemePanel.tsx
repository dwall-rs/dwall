// Responsibility: the fixed-mode control deck — what is selected, whether it is
// live, and the one commit action for this scope.
import { createSignal, Show } from "solid-js";
import { Button } from "@/components/ui/button";
import { t } from "@/i18n";
import type { Theme } from "@/models/types";
import {
  fixedStore,
  isApplied,
  isConfiguredTheme,
  scopeKey,
  toggleApply,
} from "@/store/fixed.store";
import { monitorById } from "@/store/catalog.selectors";
import { clsx } from "@/utils";
import {
  ArrowRight,
  Images,
  LoaderCircle,
  Monitor,
  Square,
} from "lucide-solid";

interface Props {
  theme: Theme;
  frameCount: number;
  onChangeTheme: () => void;
}

export function ThemePanel(props: Props) {
  const [busy, setBusy] = createSignal(false);
  const scope = () => scopeKey();
  const scopeName = () =>
    monitorById(scope())?.name ?? t("workspace.allDisplays");
  const configured = () => isConfiguredTheme(scope());
  const applied = () => isApplied(scope());
  const installed = () => !props.theme.needsDownload && props.frameCount > 0;
  const disabled = () => busy() || fixedStore.applying;

  const commit = async () => {
    if (disabled() || (!installed() && !applied())) return;
    setBusy(true);
    try {
      await toggleApply(scope());
    } finally {
      setBusy(false);
    }
  };

  const status = () => {
    if (applied()) {
      return {
        tone: "text-success",
        dot: "bg-success",
        text: t("workspace.liveOn", { name: scopeName() }),
      };
    }
    if (configured()) {
      return {
        tone: "text-muted-foreground",
        dot: "bg-muted-foreground",
        text: t("workspace.readyOn", { name: scopeName() }),
      };
    }
    return {
      tone: "text-muted-foreground",
      dot: "bg-border-2",
      text: t("workspace.previewOnly"),
    };
  };

  return (
    <section class="rounded-[16px] border border-border bg-card p-4 shadow-[0_1px_2px_rgba(0,0,0,.04)]">
      <div class="flex items-center justify-between gap-2">
        <h2 class="text-[11px] font-semibold text-muted-foreground">
          {t("workspace.theme")}
        </h2>
        <span class="flex items-center gap-1 font-mono text-[10px] text-muted-foreground">
          <Monitor class="size-3" />
          {scopeName()}
        </span>
      </div>

      <div class="mt-2.5 font-display text-[23px] font-semibold leading-tight tracking-[-0.02em]">
        {props.theme.name}
      </div>

      <div class="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 font-mono text-[10.5px] text-muted-foreground">
        <span>{t("workspace.frameCount", { n: props.frameCount })}</span>
        <span class="text-border-2">·</span>
        <span>
          {installed() ? t("themes.installed") : t("themes.notInstalled")}
        </span>
      </div>

      <div
        class={clsx(
          "mt-3 flex items-center gap-1.5 text-[11.5px]",
          status().tone,
        )}
      >
        <span class={clsx("size-1.5 rounded-full", status().dot)} />
        <span class="truncate">{status().text}</span>
      </div>

      <div class="mt-4 flex flex-col gap-2">
        <Show
          when={installed()}
          fallback={
            <>
              <Show when={applied()}>
                <Button
                  variant="destructive"
                  disabled={disabled()}
                  onClick={() => void commit()}
                >
                  <Show
                    when={busy()}
                    fallback={<Square class="size-3 fill-current" />}
                  >
                    <LoaderCircle class="size-3.5 animate-spin" />
                  </Show>
                  {fixedStore.allUnified
                    ? t("displays.stopAll")
                    : t("displays.stopHere")}
                </Button>
              </Show>
              <Button variant="outline" onClick={props.onChangeTheme}>
                <Images class="size-3.5" />
                {t("workspace.changeTheme")}
              </Button>
            </>
          }
        >
          <Button
            variant={applied() ? "destructive" : "primary"}
            disabled={disabled()}
            onClick={() => void commit()}
          >
            <Show
              when={busy()}
              fallback={
                applied() ? (
                  <Square class="size-3 fill-current" />
                ) : (
                  <ArrowRight class="size-3.5" />
                )
              }
            >
              <LoaderCircle class="size-3.5 animate-spin" />
            </Show>
            {applied()
              ? fixedStore.allUnified
                ? t("displays.stopAll")
                : t("displays.stopHere")
              : t("themes.applyTo", { name: scopeName() })}
          </Button>
          <Button variant="ghost" size="sm" onClick={props.onChangeTheme}>
            <Images class="size-3.5" />
            {t("workspace.changeTheme")}
          </Button>
        </Show>
      </div>
    </section>
  );
}
