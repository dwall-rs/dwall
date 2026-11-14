// Responsibility: the large wallpaper plate. The image is the page; chrome sits on
// its edges in translucent strips so the frame stays unobstructed.
import {
  createEffect,
  createMemo,
  createSignal,
  on,
  onCleanup,
  Show,
  untrack,
} from "solid-js";
import type { Wallpaper } from "@/models/types";
import { t } from "@/i18n";
import { ThemeThumbnail } from "@/scene/ThemeThumbnail";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { formatAngle } from "@/utils";
import { ChevronLeft, ChevronRight, Maximize2, Sparkles } from "lucide-solid";

interface Props {
  themeId: string;
  themeName: string;
  scopeName: string;
  wallpapers: Wallpaper[];
  wallpaper: Wallpaper | null;
  matchedIndex: number | null;
  onSelect: (index: number | null) => void;
}

export function PreviewPlate(props: Props) {
  const [expanded, setExpanded] = createSignal(false);
  const [displayed, setDisplayed] = createSignal<Wallpaper | null>(
    props.wallpaper,
  );
  const [outgoing, setOutgoing] = createSignal<Wallpaper | null>(null);
  const [entering, setEntering] = createSignal(true);
  let transitionTimer: ReturnType<typeof setTimeout> | undefined;

  const clearTransition = () => {
    if (transitionTimer) {
      clearTimeout(transitionTimer);
      transitionTimer = undefined;
    }
  };

  const finishTransition = () => {
    if (entering()) return;
    setEntering(true);
    clearTransition();
    transitionTimer = setTimeout(() => {
      setOutgoing(null);
      transitionTimer = undefined;
    }, 520);
  };

  // Keep the old frame underneath until the next thumbnail has decoded, then crossfade.
  // Without the decoded signal an uncached frame would flash the skeleton over the old
  // one; without the fallback timer a broken frame would leave the old one frozen.
  createEffect(
    on(
      () => props.wallpaper,
      (next) => {
        const current = untrack(displayed);
        if (next?.index === current?.index) return;

        clearTransition();
        if (!next) {
          setOutgoing(null);
          setDisplayed(null);
          setEntering(true);
          return;
        }

        if (current) {
          setOutgoing(current);
          setEntering(false);
        } else {
          setOutgoing(null);
          setEntering(true);
        }
        setDisplayed(next);
        transitionTimer = setTimeout(finishTransition, 900);
      },
    ),
  );

  onCleanup(clearTransition);

  const frameLoaded = (index: number) => {
    if (untrack(displayed)?.index === index) finishTransition();
  };

  const position = createMemo(() => {
    if (!props.wallpaper) return -1;
    return props.wallpapers.findIndex(
      (w) => w.index === props.wallpaper?.index,
    );
  });

  const step = (delta: number) => {
    const list = props.wallpapers;
    if (list.length < 2) return;
    const current = position();
    const next =
      current < 0 ? 0 : (current + delta + list.length) % list.length;
    props.onSelect(list[next].index);
  };

  return (
    <section
      aria-label={props.themeName}
      class="relative aspect-video w-full overflow-hidden rounded-[18px] border border-border bg-black shadow-[0_24px_60px_-30px_rgba(0,0,0,.55)]"
    >
      <Show
        when={displayed()}
        fallback={
          <div class="flex h-full w-full flex-col items-center justify-center gap-2 bg-card text-center">
            <span class="font-display text-[16px] font-semibold">
              {props.themeName}
            </span>
            <span class="text-[12px] text-muted-foreground">
              {t("stage.notInstalled")}
            </span>
          </div>
        }
      >
        {(wp) => (
          <>
            <div class="pointer-events-none absolute inset-0">
              <Show when={outgoing()}>
                {(old) => (
                  <div
                    class="absolute inset-0 transition-opacity duration-500 ease-out"
                    style={{ opacity: entering() ? 0 : 1 }}
                  >
                    <ThemeThumbnail
                      themeId={props.themeId}
                      index={old().index}
                      fit="cover"
                      class="h-full w-full"
                    />
                  </div>
                )}
              </Show>
              <Show when={displayed()}>
                {(current) => (
                  <div
                    class="absolute inset-0 transition-opacity duration-500 ease-out"
                    style={{ opacity: entering() ? 1 : 0 }}
                  >
                    <ThemeThumbnail
                      themeId={props.themeId}
                      index={current().index}
                      fit="cover"
                      class="h-full w-full"
                      onLoad={() => frameLoaded(current().index)}
                    />
                  </div>
                )}
              </Show>
            </div>

            {/* Edge scrims: they make the controls legible without covering the middle. */}
            <div class="pointer-events-none absolute inset-x-0 top-0 h-28 bg-linear-to-b from-black/58 to-transparent" />
            <div class="pointer-events-none absolute inset-x-0 bottom-0 h-32 bg-linear-to-t from-black/72 via-black/24 to-transparent" />

            <div class="absolute left-4 right-4 top-4 flex items-start justify-between gap-3">
              <div class="min-w-0">
                <div class="flex items-center gap-2">
                  <h2 class="truncate font-display text-[21px] font-semibold leading-tight text-white drop-shadow-sm">
                    {props.themeName}
                  </h2>
                  <Show when={wp().index === props.matchedIndex}>
                    <span class="flex shrink-0 items-center gap-1 rounded-full border border-white/20 bg-black/35 px-2 py-0.5 font-mono text-[9px] text-white/90 backdrop-blur-md">
                      <Sparkles class="size-3 text-solar" />
                      {t("stage.matched")}
                    </span>
                  </Show>
                </div>
                <div class="mt-1 truncate font-mono text-[10.5px] text-white/60">
                  {props.scopeName}
                </div>
              </div>

              <div class="flex shrink-0 items-center gap-1.5">
                <span class="rounded-full border border-white/15 bg-black/35 px-2.5 py-1 font-mono text-[10px] text-white/80 backdrop-blur-md">
                  {t("stage.altitude")} {formatAngle(wp().solar.altitude)}° ·{" "}
                  {t("stage.azimuth")} {formatAngle(wp().solar.azimuth)}°
                </span>
                <button
                  type="button"
                  aria-label={t("workspace.fullSize")}
                  onClick={() => setExpanded(true)}
                  class="grid size-7 cursor-pointer place-items-center rounded-full border border-white/15 bg-black/35 text-white/80 backdrop-blur-md transition-colors hover:bg-black/55 hover:text-white focus-visible:ring-2 focus-visible:ring-solar"
                >
                  <Maximize2 class="size-3.5" />
                </button>
              </div>
            </div>

            <Show when={props.wallpapers.length > 1}>
              <button
                type="button"
                aria-label={t("workspace.previousFrame")}
                onClick={() => step(-1)}
                class="absolute left-3 top-1/2 grid size-9 -translate-y-1/2 cursor-pointer place-items-center rounded-full border border-white/15 bg-black/30 text-white/85 backdrop-blur-md transition-colors hover:bg-black/55 focus-visible:ring-2 focus-visible:ring-solar"
              >
                <ChevronLeft class="size-5" />
              </button>
              <button
                type="button"
                aria-label={t("workspace.nextFrame")}
                onClick={() => step(1)}
                class="absolute right-3 top-1/2 grid size-9 -translate-y-1/2 cursor-pointer place-items-center rounded-full border border-white/15 bg-black/30 text-white/85 backdrop-blur-md transition-colors hover:bg-black/55 focus-visible:ring-2 focus-visible:ring-solar"
              >
                <ChevronRight class="size-5" />
              </button>
            </Show>

            <div class="absolute inset-x-4 bottom-4 flex items-end justify-between gap-4">
              <div class="min-w-0">
                <div class="font-mono text-[10.5px] text-white/65">
                  {t("workspace.frameOf", {
                    current: position() + 1,
                    total: props.wallpapers.length,
                  })}
                </div>
                <div class="mt-1.5 h-[3px] w-36 overflow-hidden rounded-full bg-white/20">
                  <div
                    class="h-full rounded-full bg-solar transition-[width] duration-300"
                    style={{
                      width: `${((position() + 1) / Math.max(1, props.wallpapers.length)) * 100}%`,
                    }}
                  />
                </div>
              </div>
            </div>
          </>
        )}
      </Show>

      <Dialog open={expanded()} onOpenChange={setExpanded}>
        <DialogContent
          class="w-[min(93vw,1180px)] max-w-none! border-white/15 bg-[#070b10] p-0 text-white sm:max-w-none!"
          showCloseButton
        >
          <DialogTitle class="sr-only">{props.themeName}</DialogTitle>
          <div class="p-4">
            <div class="mb-3 flex items-center justify-between gap-3 pr-8">
              <div class="min-w-0">
                <div class="truncate font-display text-[15px] font-semibold">
                  {props.themeName}
                </div>
                <div class="mt-0.5 font-mono text-[10.5px] text-white/55">
                  {props.scopeName}
                  <Show when={props.wallpaper}>
                    {(wp) =>
                      ` · ${t("stage.altitude")} ${formatAngle(wp().solar.altitude)}° · ${t("stage.azimuth")} ${formatAngle(wp().solar.azimuth)}°`
                    }
                  </Show>
                </div>
              </div>
            </div>
            <div class="flex max-h-[78vh] min-h-[46vh] items-center justify-center overflow-hidden rounded-xl bg-black">
              <Show when={props.wallpaper}>
                {(wp) => (
                  <ThemeThumbnail
                    themeId={props.themeId}
                    index={wp().index}
                    fit="contain"
                    class="max-h-[76vh] w-full"
                  />
                )}
              </Show>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </section>
  );
}
