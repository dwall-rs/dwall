import { createEffect, createMemo, createRoot, createSignal } from "solid-js";
import * as i18n from "@solid-primitives/i18n";

import { loadFonts } from "@/utils";

import * as enUS from "./en-US";
import * as zhCN from "./zh-CN";
import * as zhHK from "./zh-HK";
import * as zhTW from "./zh-TW";
import * as jaJP from "./ja-JP";
import * as koKR from "./ko-KR";

export type Locale = "en-US" | "zh-CN" | "ja-JP" | "zh-HK" | "zh-TW" | "ko-KR";
export type RawDictionary = typeof enUS.dict;

export const LANGUAGES = {
  "en-US": "English",
  "zh-CN": "简体中文",
  "ja-JP": "日本語",
  "zh-HK": "繁體中文（香港）",
  "zh-TW": "正體中文（台灣）",
  "ko-KR": "한국어",
} as const satisfies Record<Locale, string>;

/** Locale → font/line-height scheme (matches the [data-lang] rules in styles/i18n.css). */
const DATA_LANG = {
  "en-US": "en",
  "zh-CN": "zh-hans",
  "zh-HK": "zh-hant",
  "zh-TW": "zh-hant",
  "ja-JP": "ja",
  "ko-KR": "ko",
} as const satisfies Record<Locale, string>;

const dictionaries: Record<Locale, RawDictionary> = {
  "en-US": enUS.dict,
  "zh-CN": zhCN.dict,
  "ja-JP": jaJP.dict,
  "zh-HK": zhHK.dict,
  "zh-TW": zhTW.dict,
  "ko-KR": koKR.dict,
};

const getInitialLocale = (): Locale => {
  if (typeof window === "undefined") return "en-US";

  const savedLocale = localStorage.getItem("locale") as Locale;
  if (savedLocale) return savedLocale;

  const browserLang = navigator.language;
  if (browserLang in LANGUAGES) return browserLang as Locale;

  return "en-US";
};

const [locale, setLocale] = createSignal<Locale>(getInitialLocale());

// Module-level singleton: owned by createRoot so the memo/effect aren't created outside a reactive root.
const dict = createRoot(() =>
  createMemo(() => i18n.flatten(dictionaries[locale()])),
);

export const t = i18n.translator(dict, i18n.resolveTemplate);

createRoot(() => {
  createEffect(() => {
    const current = locale();
    localStorage.setItem("locale", current);
    const el = document.documentElement;
    el.lang = current;
    // Activate the [data-lang] font/line-height/direction rules in styles/i18n.css
    el.setAttribute("data-lang", DATA_LANG[current]);
    // Load the Google Fonts family backing the active [data-lang] rules.
    loadFonts([DATA_LANG[current]]);
  });
});

export { locale, setLocale };
