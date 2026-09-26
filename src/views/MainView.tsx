/* ===== src/components/views/MainView.tsx ===== */
// 职责：主视图网格分档 + 主题库抽屉形态切换；backdrop 仅在最小档且展开时存在。
//       固定模式：显示器作用域 + 舞台（+ 主题库）；随机模式无左栏（池在主题库中编辑）。
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
        // relative：让抽屉的定位祖先就是本网格，从而被下面的 overflow-hidden 正确裁剪，
        // 否则 translate-x-full 的抽屉会溢出视口产生横向滚动条。
        "relative grid h-full overflow-hidden",
        // 单行，且行高严格等于容器高度：内容超高时收缩而不是撑破视口
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

      {/* 最小档 backdrop：点击收起抽屉；xl+ 库为内联栏，无需遮罩 */}
      <Show when={uiStore.libOpen}>
        <button
          type="button"
          aria-label={t("library.toggleOpen")}
          onClick={() => setLibOpen(false)}
          class="absolute inset-0 z-20 cursor-default xl:hidden"
        />
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
