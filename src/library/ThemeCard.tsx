// Responsibility: one theme in the library. The image, name, install state and pool
// state are all visible at once; only the download control sits above the tile's own
// selection target.
import { Show } from "solid-js";
import { t } from "@/i18n";
import { themeById } from "@/store/catalog.selectors";
import { modeStore } from "@/store/mode.store";
import {
  fixedStore,
  isApplied,
  scopeKey,
  setMonitorTheme,
} from "@/store/fixed.store";
import { randomStore, toggle } from "@/store/random.store";
import { ThemeThumbnail } from "@/scene/ThemeThumbnail";
import { ThemeDownloadButton } from "@/scene/ThemeDownloadButton";
import { clsx } from "@/utils";
import { Check } from "lucide-solid";

interface Props {
  id: string;
}

export function ThemeCard(props: Props) {
  const theme = () => themeById(props.id);
  const fixed = () => modeStore.mode === "fixed";
  const needsDownload = () => theme().needsDownload;

  const selected = () =>
    fixed()
      ? fixedStore.monitorThemes[scopeKey()] === props.id
      : randomStore.selected.includes(props.id);
  const applied = () => fixed() && isApplied(scopeKey()) && selected();

  const status = () => {
    if (needsDownload()) return t("themes.notInstalled");
    if (applied()) return t("themes.applied");
    if (selected()) {
      return fixed() ? t("themes.selected") : t("themes.inPool");
    }
    return fixed() ? t("themes.installed") : t("themes.outPool");
  };

  const onClick = () => {
    if (needsDownload()) return;
    if (fixed()) {
      if (fixedStore.monitorThemes[scopeKey()] !== props.id) {
        setMonitorTheme(scopeKey(), props.id);
      }
    } else {
      toggle(props.id);
    }
  };

  return (
    <div
      class={clsx(
        "group relative overflow-hidden rounded-[14px] border bg-card transition-colors",
        "has-[:focus-visible]:ring-3 has-[:focus-visible]:ring-ring/50",
        selected()
          ? fixed()
            ? "border-primary/60"
            : "border-warning/60"
          : "border-border hover:border-border-2",
      )}
    >
      <button
        type="button"
        aria-label={theme().name}
        aria-pressed={selected()}
        disabled={needsDownload()}
        onClick={onClick}
        class={clsx(
          "absolute inset-0 z-10 rounded-[14px] outline-none",
          needsDownload() ? "cursor-default" : "cursor-pointer",
        )}
      />

      <div class="relative aspect-[16/10] overflow-hidden bg-muted">
        <ThemeThumbnail
          themeId={props.id}
          class="transition-transform duration-500 group-hover:scale-[1.025]"
        />
        <div class="pointer-events-none absolute inset-x-0 top-0 h-14 bg-linear-to-b from-black/35 to-transparent" />

        <Show when={selected()}>
          <span
            class={clsx(
              "absolute right-2 top-2 grid size-5 place-items-center rounded-full text-white shadow-sm",
              fixed() ? "bg-primary" : "bg-warning text-warning-foreground",
            )}
          >
            <Check class="size-3" strokeWidth={3} />
          </span>
        </Show>

        <Show when={applied()}>
          <span class="absolute left-2 top-2 rounded-full bg-black/45 px-2 py-0.5 font-mono text-[9px] text-white/85 backdrop-blur-md">
            {t("themes.applied")}
          </span>
        </Show>

        <Show when={needsDownload()}>
          <div class="absolute bottom-2 right-2 z-20">
            <ThemeDownloadButton themeId={props.id} />
          </div>
        </Show>
      </div>

      <div class="flex items-center justify-between gap-2 px-3 py-2.5">
        <div class="min-w-0">
          <div class="truncate text-[12.5px] font-semibold leading-tight">
            {theme().name}
          </div>
          <div
            class={clsx(
              "mt-0.5 truncate font-mono text-[9.5px]",
              applied()
                ? "text-success"
                : needsDownload()
                  ? "text-muted-foreground"
                  : selected()
                    ? fixed()
                      ? "text-primary"
                      : "text-warning"
                    : "text-muted-foreground",
            )}
          >
            {status()}
          </div>
        </div>
      </div>
    </div>
  );
}
