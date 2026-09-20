/* ===== src/components/views/MainView.tsx ===== */
// 职责：主视图网格分档 + 主题库抽屉形态切换；backdrop 仅在最小档且展开时存在。
import { uiStore, setLibOpen } from "@/store/ui.store";
import { StageColumn } from "@/stage/StageColumn";
import { LibraryColumn } from "@/library/LibraryColumn";
import { clsx } from "@/utils";
import { ScopeColumn } from "~/scope/ScopeColumn";
import { Show } from "solid-js";
import { Button } from "~/components/ui/button";

export function MainView() {
  return (
    <div
      class={clsx(
        "grid h-full",
        // 最小档：两栏（库为抽屉，脱离流）；标准/宽屏：三栏
        "grid-cols-[220px_minmax(0,1fr)]",
        "xl:grid-cols-[240px_minmax(0,1fr)_320px]",
        "2xl:grid-cols-[266px_minmax(0,1fr)_372px]",
      )}
    >
      <ScopeColumn />
      <StageColumn />

      {/* 最小档 backdrop：点击收起抽屉；xl+ 不渲染交互 */}
      <Show when={uiStore.libOpen}>
        <Button aria-label="收起主题库" onClick={() => setLibOpen(false)} />
      </Show>

      {/* 主题库：最小档=右侧抽屉，xl+=内联第三栏（同一实例） */}
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
