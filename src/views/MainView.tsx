/* ===== src/components/views/MainView.tsx ===== */
// Responsibility: Main-view grid tiers + theme library drawer/inline switching; the backdrop exists only at the smallest tier while expanded.
//       Fixed mode: monitor scope + stage (+ theme library); random mode has no left column (the pool is edited in the theme library).
import { uiStore, setLibOpen } from "@/store/ui.store";
import { StageColumn } from "@/stage/StageColumn";
import { LibraryColumn } from "@/library/LibraryColumn";
import { t } from "@/i18n";
import { clsx } from "@/utils";
import { ScopeColumn } from "~/scope/ScopeColumn";
import { Show } from "solid-js";

export function MainView() {
  const fixed = () => uiStore.mode === "fixed";

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
        class={clsx(
          "absolute inset-y-0 right-0 z-30 w-85 transform bg-background shadow-[-24px_0_60px_rgba(0,0,0,.4)] transition-transform duration-300 ease-[cubic-bezier(.22,.61,.36,1)]",
          uiStore.libOpen ? "translate-x-0" : "translate-x-full",
          "xl:static xl:z-auto xl:w-auto xl:translate-x-0 xl:transform-none xl:bg-transparent xl:shadow-none xl:transition-none",
        )}
      >
        <LibraryColumn />
      </div>
    </div>
  );
}
