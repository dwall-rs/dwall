import * as i18n from "@solid-primitives/i18n";

import * as enUS from "./en-US";
import { createResource, createSignal } from "solid-js";

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

export async function fetchDictionary(locale: Locale): Promise<Dictionary> {
  const dict: RawDictionary = (await import(`../i18n/${locale}.ts`)).dict;
  return i18n.flatten(dict);
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

const [dict] = createResource(locale, fetchDictionary, {
  initialValue: i18n.flatten(enUS.dict),
});

export const t = i18n.translator(dict, i18n.resolveTemplate);

export { locale, setLocale };
