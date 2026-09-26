import { createEffect, onCleanup } from "solid-js";

// ─── 字体配置表 ────────────────────────────────────────────────
const FONT_MAP = {
  en: { family: "Inter", weights: "400;500;700" },
  fr: { family: "Inter", weights: "400;500;700" },
  de: { family: "Inter", weights: "400;500;700" },
  ru: { family: "Noto+Sans", weights: "400;500;700" },
  "zh-hans": { family: "Noto+Sans+SC", weights: "400;500;700" },
  "zh-hant": { family: "Noto+Sans+TC", weights: "400;500;700" },
  ja: { family: "Noto+Sans+JP", weights: "400;500;700" },
  ko: { family: "Noto+Sans+KR", weights: "400;500;700" },
  hi: { family: "Noto+Sans+Devanagari", weights: "400;500;700" },
  th: { family: "Noto+Sans+Thai", weights: "400;500;700" },
  bn: { family: "Noto+Sans+Bengali", weights: "400;500;700" },
  ar: { family: "Noto+Sans+Arabic", weights: "400;500;700" },
  he: { family: "Noto+Sans+Hebrew", weights: "400;500;700" },
  fa: { family: "Noto+Sans+Arabic", weights: "400;500;700" },
} as const satisfies Record<string, { family: string; weights: string }>;

export type Lang = keyof typeof FONT_MAP;

// ─── 已加载字体缓存（模块级单例，避免重复插入） ──────────────
const loaded = new Set<string>();

function filterValidLangs(langs: string[]): Lang[] {
  return langs.filter((lang): lang is Lang => {
    if (lang in FONT_MAP) return true;
    if (!import.meta.env.PROD) {
      console.warn(
        `[useFontLoader] unsupported lang: "${lang}". ` +
          `Supported values: ${Object.keys(FONT_MAP).join(", ")}`,
      );
    }
    return false;
  });
}

export interface UseFontLoaderOptions {
  /**
   * 要加载的字体对应的语言
   *
   * 默认为`en`
   */
  langs?: Lang | Lang[];

  /**
   * 指定 data-lang 写入的目标元素。
   * - 传入 ref：仅在该元素上生效，适合局部语言切换。
   * - 不传（undefined）：写入 ，全局生效。
   * 卸载时自动恢复目标元素的原始 data-lang 值。
   *
   * apply 为 false 时此选项无效。
   */
  targetRef?: HTMLElement;

  /**
   * 是否将 data-lang 写入目标元素以应用字体样式。
   * - true（默认）：写入 data-lang，字体立即生效。
   * - false：仅预加载字体资源，不修改任何 DOM，
   *   适合提前预热用户可能切换到的语言字体。
   * @default
   */
  apply?: boolean;
}

/* ----------------------------------------------------------------
   工具：插入 Google Fonts
   ---------------------------------------------------------------- */
function loadGoogleFonts(langs: Lang[]): void {
  const toLoad = langs.filter((lang) => !loaded.has(FONT_MAP[lang].family));
  if (toLoad.length === 0) return;

  const families = [
    ...new Set(
      toLoad.map((lang) => {
        const { family, weights } = FONT_MAP[lang];
        return `family=${family}:wght@${weights}`;
      }),
    ),
  ];

  const link = document.createElement("link");
  link.rel = "stylesheet";
  link.href = `https://fonts.googleapis.com/css2?${families.join("&")}&display=swap`;

  document.head.appendChild(link);
  for (const lang of toLoad) {
    loaded.add(FONT_MAP[lang].family);
  }
}

// ─── Hook ─────────────────────────────────────────────────────
export function useFontLoader(options?: UseFontLoaderOptions) {
  const {
    langs = "en",
    targetRef = document.documentElement,
    apply = true,
  } = options ?? {};

  const langList = Array.isArray(langs) ? langs : [langs];
  const validLangs = filterValidLangs(langList);

  // --- 加载字体资源 ---
  createEffect(() => {
    loadGoogleFonts(validLangs);
  });

  // --- 将 data-lang 写入目标元素 ---
  createEffect(() => {
    if (!apply || validLangs.length === 0) return;

    // targetRef 未传或 ref.current 尚未挂载时，回退到
    const el = targetRef ?? document.documentElement;
    const lang = validLangs[0];
    const prev = el.getAttribute("data-lang");

    el.setAttribute("data-lang", lang);

    onCleanup(() => {
      // 卸载时恢复：原本有值则还原，原本无则移除
      if (prev === null) {
        el.removeAttribute("data-lang");
      } else {
        el.setAttribute("data-lang", prev);
      }
    });
  });
}
