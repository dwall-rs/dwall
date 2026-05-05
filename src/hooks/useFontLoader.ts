import { createEffect, onCleanup } from "solid-js";
import { locale, type Locale } from "~/i18n";

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

export interface UseFontLoaderOptions {
  /**
   * 是否将 data-lang 写入目标元素以应用字体样式。
   * - true（默认）：写入 data-lang，字体立即生效。
   * - false：仅预加载字体资源，不修改任何 DOM，
   *   适合提前预热用户可能切换到的语言字体。
   * @default
   */
  apply?: boolean;
}

function localeToLang(locale: Locale): Lang {
  const lang = (
    locale === "zh-CN"
      ? "zh-hans"
      : locale === "zh-HK" || locale === "zh-TW"
        ? "zh-hant"
        : locale.split("-")[0]
  ) as Lang;

  if (!(lang in FONT_MAP)) {
    if (!import.meta.env.PROD) {
      console.warn(
        `[useFontLoader] unsupported lang: "${lang}". ` +
          `Supported values: ${Object.keys(FONT_MAP).join(", ")}`,
      );
    }
    return "en";
  }
  return lang;
}

function loadGoogleFonts(locale: Locale): void {
  const lang = localeToLang(locale);

  if (loaded.has(FONT_MAP[lang].family)) return;

  const { family, weights } = FONT_MAP[lang];
  const f = `family=${family}:wght@${weights}`;

  const link = document.createElement("link");
  link.rel = "stylesheet";
  link.href = `https://fonts.googleapis.com/css2?${f}&display=swap`;

  document.head.appendChild(link);

  loaded.add(FONT_MAP[lang].family);
}

// ─── Hook ─────────────────────────────────────────────────────
export function useFontLoader(options?: UseFontLoaderOptions) {
  locale;

  const { apply = true } = options ?? {};

  // --- 加载字体资源 ---
  createEffect(() => {
    loadGoogleFonts(locale());
  });

  // --- 将 data-lang 写入目标元素 ---
  createEffect(() => {
    if (!apply) return;

    const lang = localeToLang(locale());

    const el = document.documentElement;
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
