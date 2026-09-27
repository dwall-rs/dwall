import { logger } from "./logger";

const log = logger.child("fonts");

/**
 * Google Fonts family per `data-lang` value.
 *
 * Keys mirror the `[data-lang]` bindings in `styles/i18n.css`; each family must stay
 * in sync with the corresponding `--font-*` token there.
 */
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

export type FontLang = keyof typeof FONT_MAP;

// Families already requested in this session; each is inserted at most once so that
// switching languages only appends the newly needed `<link>`.
const loadedFamilies = new Set<string>();

/** Insert a Google Fonts `<link>` for every not-yet-loaded family in `langs`. */
export function loadFonts(langs: readonly string[]): void {
  const families = new Set<string>();

  for (const lang of langs) {
    const entry = FONT_MAP[lang as FontLang] as
      | { family: string; weights: string }
      | undefined;
    if (!entry) {
      log.warn(
        `unsupported lang: "${lang}". Supported values: ${Object.keys(FONT_MAP).join(", ")}`,
      );
      continue;
    }
    if (loadedFamilies.has(entry.family)) continue;
    loadedFamilies.add(entry.family);
    families.add(`family=${entry.family}:wght@${entry.weights}`);
  }

  if (families.size === 0) return;

  const link = document.createElement("link");
  link.rel = "stylesheet";
  link.href = `https://fonts.googleapis.com/css2?${[...families].join("&")}&display=swap`;
  document.head.appendChild(link);
}
