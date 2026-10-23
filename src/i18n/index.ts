import { createEffect, createMemo, createRoot, createSignal } from "solid-js";
import * as i18n from "@solid-primitives/i18n";

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

const STORAGE_KEY = "locale";

/**
 * The supported locale a browser tag asks for, or undefined when there is none.
 *
 * A browser reports "ja", "ko" or "zh-CN" as readily as "ja-JP" and "zh-Hans-CN", and
 * accepting only an exact tag meant a Japanese or Korean system fell back to English and a
 * bare "zh" was ignored. The language subtag is what decides the dictionary; for Chinese
 * the script or the region decides the variant, because Simplified and Traditional are not
 * interchangeable. `maximize` fills in the region a bare tag leaves out: "zh" becomes
 * "zh-Hans-CN" and "zh-Hant" becomes "zh-Hant-TW".
 */
const matchLocale = (tag: string): Locale | undefined => {
  let parsed: Intl.Locale;
  try {
    parsed = new Intl.Locale(tag);
  } catch {
    return undefined; // A malformed tag: there is nothing to match against.
  }

  switch (parsed.language) {
    case "en":
      return "en-US";
    case "ja":
      return "ja-JP";
    case "ko":
      return "ko-KR";
    case "zh":
      break;
    default:
      return undefined;
  }

  const region = parsed.region ?? parsed.maximize().region;
  if (region === "TW") return "zh-TW";
  // Macau writes Traditional characters too, and its closest dictionary is Hong Kong's.
  if (region === "HK" || region === "MO") return "zh-HK";
  return "zh-CN";
};

/** The stored choice, or undefined when there is none or storage cannot be read. */
const readSavedLocale = (): Locale | undefined => {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    // Validate the persisted value: a stale key from an older version would make
    // `dictionaries[locale()]` undefined and crash during module init (blank app).
    if (saved && saved in LANGUAGES) return saved as Locale;
  } catch {
    // Storage can be unavailable — private mode, a blocked origin — and the browser's
    // language is a perfectly good answer. Throwing here would take the whole module,
    // and with it the app.
  }
  return undefined;
};

const getInitialLocale = (): Locale => {
  if (typeof window === "undefined") return "en-US";

  const savedLocale = readSavedLocale();
  if (savedLocale) return savedLocale;

  // Every language the user asked for, most preferred first: `navigator.language` alone
  // is the first entry of this list.
  const preferred = navigator.languages?.length
    ? navigator.languages
    : [navigator.language];
  for (const tag of preferred) {
    const matched = matchLocale(tag);
    if (matched) return matched;
  }

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
    try {
      localStorage.setItem(STORAGE_KEY, current);
    } catch {
      // Not fatal: the choice simply does not survive a restart.
    }
    const el = document.documentElement;
    el.lang = current;
    // Activate the [data-lang] font/line-height/direction rules in styles/i18n.css.
    // The families those rules name come from the system font stack: the app used
    // to fetch webfonts from a third-party CDN at startup, which conflicted with
    // both the "no data transmission" documentation and the low-footprint goal
    // (a CJK family alone is megabytes, and is unreachable in some regions).
    el.setAttribute("data-lang", DATA_LANG[current]);
  });
});

export { locale, matchLocale, setLocale };
