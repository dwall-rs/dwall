// Responsibility: A single theme tile; addition — at the smallest tier in fixed mode the drawer collapses automatically after selection so the stage feedback stays visible.
import { setLibOpen } from "@/store/ui.store";
import { modeStore } from "@/store/mode.store";
import { fixedStore, scopeKey, setMonitorTheme } from "@/store/fixed.store";
import { randomStore, toggle } from "@/store/random.store";
import { useMediaQuery } from "@/hooks/useMediaQuery";
import { MQ_XL } from "@/lib/layout";
import { themeById } from "@/store/catalog.selectors";
import { t } from "@/i18n";
import { Show } from "solid-js";
import { clsx } from "@/utils";
import { Check } from "lucide-solid";
import { ThemeThumbnail } from "~/scene/ThemeThumbnail";
import { ThemeDownloadButton } from "~/scene/ThemeDownloadButton";

interface Props {
  id: string;
  index: number;
}

export function ThemeTile(props: Props) {
  const isXl = useMediaQuery(() => MQ_XL);

  // Read through a function: a download marks the theme installed in the catalog store, and the tile has to notice.
  const theme = () => themeById(props.id);
  /** Nothing that is not on disk can be applied, so it cannot be selected either — the button below installs it first. */
  const selectable = () => !theme().needsDownload;
  const on = () =>
    selectable() &&
    (modeStore.mode === "fixed"
      ? fixedStore.monitorThemes[scopeKey()] === props.id
      : randomStore.selected.includes(props.id));

  const onClick = () => {
    if (!selectable()) return;

    if (modeStore.mode === "fixed") {
      setMonitorTheme(scopeKey(), props.id);
      // Smallest tier: collapse the drawer right after assigning, so the user immediately sees the sun-path/preview update; random mode stays open for consecutive checking
      if (!isXl()) setLibOpen(false);
    } else {
      toggle(props.id);
    }
  };

  return (
    <div
      title={selectable() ? undefined : t("stage.notInstalled")}
      style={{ "animation-delay": `${props.index * 40}ms` }}
      class={clsx(
        "group relative aspect-16/10 overflow-hidden rounded-lg border transition-all duration-200 animate-rise",
        // The ring is drawn on the tile rather than on the button inside it: the tile
        // clips its contents, so a ring around the button would be cut off.
        "has-[:focus-visible]:border-ring has-[:focus-visible]:ring-3 has-[:focus-visible]:ring-ring/50",
        selectable()
          ? "hover:-translate-y-1 hover:scale-[1.02] hover:border-border-2 hover:shadow-[0_12px_26px_rgba(0,0,0,.5)]"
          : "cursor-default",
        on() && modeStore.mode === "fixed" && "border-primary",
        on() && modeStore.mode === "random" && "border-warning",
      )}
    >
      {/* The tile selects a theme and can carry a download button of its own, so the
          click target is a button laid over the tile rather than the tile itself: a
          button may not contain another one, and this is what the keyboard reaches. */}
      <button
        type="button"
        aria-label={theme().name}
        aria-pressed={on()}
        disabled={!selectable()}
        onClick={onClick}
        class={clsx(
          "absolute inset-0 z-10 rounded-lg",
          selectable() ? "cursor-pointer" : "cursor-default",
        )}
      />
      <ThemeThumbnail
        themeId={props.id}
        class="transition-transform duration-500 group-hover:scale-105"
      />
      <div class="absolute inset-x-0 bottom-0 bg-linear-to-t from-black/72 to-transparent px-2 pb-1.5 pt-3.5 font-display text-[10.5px] font-semibold text-white opacity-0 transition-opacity group-hover:opacity-100">
        {theme().name}
      </div>
      <span
        class={clsx(
          "absolute right-1.5 top-1.5 grid size-5 place-items-center rounded-md transition-all duration-200",
          on() ? "scale-100 opacity-100" : "scale-50 opacity-0",
          modeStore.mode === "fixed"
            ? "bg-primary text-primary-foreground"
            : "bg-warning text-white",
        )}
      >
        <Check class="size-3" strokeWidth={3} />
      </span>
      {/* A theme that is not on disk is offered for download right here, so both modes
          install themes from the same place instead of only the fixed-mode stage. */}
      <Show when={theme().needsDownload}>
        <div class="absolute bottom-1.5 right-1.5 z-20">
          <ThemeDownloadButton themeId={props.id} />
        </div>
      </Show>
    </div>
  );
}
