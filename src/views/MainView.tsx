// Responsibility: Main-view grid tiers + theme library drawer/inline switching; the backdrop exists only at the smallest tier while expanded.
//       Fixed mode: monitor scope + stage (+ theme library); random mode has no left column (the pool is edited in the theme library).
import { uiStore, setLibOpen } from "@/store/ui.store";
import { modeStore } from "@/store/mode.store";
import { StageColumn } from "@/stage/StageColumn";
import { LibraryColumn } from "@/library/LibraryColumn";
import { t } from "@/i18n";
import { LIBRARY_TOGGLE_ID, MQ_XL } from "@/lib/layout";
import { useMediaQuery } from "@/hooks/useMediaQuery";
import { clsx } from "@/utils";
import { ScopeColumn } from "~/scope/ScopeColumn";
import { createEffect, on, Show } from "solid-js";

export function MainView() {
  const fixed = () => modeStore.mode === "fixed";
  const isXl = useMediaQuery(MQ_XL);
  let drawer: HTMLDivElement | undefined;

  // The drawer covers the page, so it has to behave like one: opening it moves focus
  // inside, and closing it — with the toggle, the backdrop, or by picking a theme — puts
  // focus back on the toggle. Without this, Tab walked the page behind the drawer and
  // closing dropped focus on <body>, so a keyboard user lost their place.
  //
  // The move is deferred to a microtask because the class that makes the drawer visible is
  // applied by an effect created after this one, and an invisible element cannot be
  // focused.
  createEffect(
    on(
      () => uiStore.libOpen,
      (open) => {
        // At xl the library is an inline column and libOpen is ignored there, so there is
        // neither a drawer to move into nor a toggle to come back to.
        if (isXl()) return;
        queueMicrotask(() => {
          if (open) {
            drawer?.focus();
            return;
          }
          document.getElementById(LIBRARY_TOGGLE_ID)?.focus();
        });
      },
      { defer: true },
    ),
  );

  return (
    <div
      class={clsx(
        // relative: makes this grid the drawer's positioning ancestor, so it is properly clipped by the overflow-hidden below,
        // otherwise a translate-x-full drawer would overflow the viewport and produce a horizontal scrollbar.
        "relative grid h-full overflow-hidden",
        // single row, with the row height exactly equal to the container height: shrinks instead of bursting the viewport when content overflows
        "grid-rows-[minmax(0,1fr)]",
        fixed()
          ? "grid-cols-[220px_minmax(0,1fr)] xl:grid-cols-[240px_minmax(0,1fr)_320px] 2xl:grid-cols-[266px_minmax(0,1fr)_372px]"
          : "grid-cols-[minmax(0,1fr)] xl:grid-cols-[minmax(0,1fr)_320px] 2xl:grid-cols-[minmax(0,1fr)_372px]",
      )}
    >
      <Show when={fixed()}>
        <ScopeColumn />
      </Show>
      <StageColumn />

      {/* Smallest-tier backdrop: click to collapse the drawer; at xl+ the library is an inline column, no scrim needed */}
      <Show when={uiStore.libOpen}>
        <button
          type="button"
          aria-label={t("library.toggleOpen")}
          onClick={() => setLibOpen(false)}
          class="absolute inset-0 z-20 cursor-default xl:hidden"
        />
      </Show>

      {/* Theme library: smallest tier = right drawer, xl+ = inline third column (same instance) */}
      <div
        ref={drawer}
        tabindex="-1"
        class={clsx(
          // outline-none: this element only ever takes focus as the drawer's landing
          // point, so it must not paint a ring of its own.
          "absolute inset-y-0 right-0 z-30 w-85 transform bg-background shadow-[-24px_0_60px_rgba(0,0,0,.4)] outline-none transition-[transform,visibility] duration-300 ease-[cubic-bezier(.22,.61,.36,1)]",
          // Closed means off-screen, and off-screen still takes focus: `invisible` is
          // what keeps Tab out of a drawer nobody can see. At xl this same element is
          // an inline column, so it stays visible there whatever the drawer state is.
          uiStore.libOpen
            ? "visible translate-x-0"
            : "invisible translate-x-full",
          "xl:visible xl:static xl:z-auto xl:w-auto xl:translate-x-0 xl:transform-none xl:bg-transparent xl:shadow-none xl:transition-none",
        )}
      >
        <LibraryColumn />
      </div>
    </div>
  );
}
