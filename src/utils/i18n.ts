export const translate = (
  translations: Translations,
  key: TranslationKey,
  params: Record<string, string> = {},
) => {
  const translation = translations[key];
  if (!translation) return key;

  if (typeof translation === "string") {
    return translation;
  }

  let result = translation.template;
  for (const param of translation.params) {
    result = result.replace(`{{${param}}}`, params[param] || "");
  }
  return result;
};

export const translateErrorMessage = (
  translations: Translations,
  key: TranslationKey,
  error: unknown,
  params: Record<string, string> = {},
) => {
  return translate(translations, key, { ...params, error: String(error) });
};
