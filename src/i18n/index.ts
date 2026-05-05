import { createEffect, createMemo, createSignal } from "solid-js";
import * as i18n from "@solid-primitives/i18n";

import * as enUS from "./en-US";
import * as zhCN from "./zh-CN";
import * as zhHK from "./zh-HK";
import * as zhTW from "./zh-TW";
import * as jaJP from "./ja-JP";
import * as koKR from "./ko-KR";

export type Locale = "en-US" | "zh-CN" | "ja-JP" | "zh-HK" | "zh-TW" | "ko-KR";
export type RawDictionary = typeof enUS.dict;
export type Dictionary = i18n.Flatten<RawDictionary>;

export const LANGUAGES = {
  "en-US": "English",
  "zh-CN": "简体中文",
  "ja-JP": "日本語",
  "zh-HK": "繁體中文（香港）",
  "zh-TW": "正體中文（台湾）",
  "ko-KR": "한국어",
} as const satisfies Record<Locale, string>;

const dictionaries: Record<Locale, RawDictionary> = {
  "en-US": enUS.dict,
  "zh-CN": zhCN.dict,
  "ja-JP": jaJP.dict,
  "zh-HK": zhHK.dict,
  "zh-TW": zhTW.dict,
  "ko-KR": koKR.dict,
};

export function fetchDictionary(locale: Locale): Dictionary {
  return i18n.flatten(dictionaries[locale]);
}

const getInitialLocale = (): Locale => {
  if (typeof window === "undefined") return "en-US";

  const savedLocale = localStorage.getItem("locale") as Locale;
  if (savedLocale) return savedLocale;

  const browserLang = navigator.language;
  if (browserLang in LANGUAGES) return browserLang as Locale;

  return "en-US";
};

const [locale, setLocale] = createSignal<Locale>(getInitialLocale());

const dict = createMemo(() => i18n.flatten(dictionaries[locale()]));

export const t = i18n.translator(dict, i18n.resolveTemplate);

createEffect(() => {
  localStorage.setItem("locale", locale());
  document.documentElement.lang = locale();
});

export { locale, setLocale };
