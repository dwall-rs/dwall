/* ===== src/lib/layout.ts ===== */
// 职责：布局体系的单一事实源——最小窗口尺寸 + 媒体查询串，避免魔法数字散落。
export const MIN_WINDOW = { width: 1024, height: 640 } as const;
/** 标准档起点：主题库从抽屉变回内联第三栏。 */
export const MQ_XL = "(min-width: 1280px)";
/** 最小窗口宽度查询（低于则提示）。 */
export const MQ_MIN = `(min-width: ${MIN_WINDOW.width}px)`;
/** 低高度窗口：隐藏日轨弧，把高度让给预览。 */
export const MQ_SHORT = "(max-height: 720px)";
