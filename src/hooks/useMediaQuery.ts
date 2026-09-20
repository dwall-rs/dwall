/* ===== src/hooks/useMediaQuery.ts ===== */
// 职责：订阅一条媒体查询并返回布尔；组件据此做「无法纯 CSS 表达」的决策（如抽屉自动关闭）。

import { createEffect, createSignal } from "solid-js";

export function useMediaQuery(query: string): Accessor<boolean> {
  const [matches, setMatches] = createSignal(
    typeof window !== "undefined" && window.matchMedia(query).matches,
  );
  createEffect(() => {
    const mql = window.matchMedia(query);
    const onChange = () => setMatches(mql.matches);
    onChange();
    mql.addEventListener("change", onChange);
    return () => mql.removeEventListener("change", onChange);
  }, [query]);

  return matches;
}
