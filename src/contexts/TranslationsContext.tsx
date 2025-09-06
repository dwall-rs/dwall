import type { JSX } from "solid-js";
import { createContext, useContext, createResource } from "solid-js";
import { getTranslations } from "~/commands";
import { translateErrorMessage as translateErrorMessageFn } from "~/utils/i18n";

type TranslationsContextType = {
  translations: () => Translations | undefined;
  translate: (
    key: TranslationKey,
    params?: Record<string, string | JSX.Element>,
  ) => string | JSX.Element[];
  translateErrorMessage: (
    key: TranslationKey,
    error: unknown,
    params?: Record<string, string>,
  ) => string;
};

const TranslationsContext = createContext<TranslationsContextType>();

export const TranslationsProvider = (props: { children: JSX.Element }) => {
  const [translations] = createResource(getTranslations);

  const translate = (
    key: TranslationKey,
    params: Record<string, string | JSX.Element> = {},
  ) => {
    const translationData = translations();

    if (!translationData) return key;

    const translation = translationData[key];
    if (!translation) return key;

    if (typeof translation === "string") {
      return translation;
    }

    const hasComponentParams = Object.values(params).some(
      (value) => typeof value !== "string",
    );

    if (hasComponentParams) {
      const elements: JSX.Element[] = [];
      let template = translation.template;

      for (const param of translation.params) {
        const value = params[param];
        const placeholder = `{{${param}}}`;
        const parts = template.split(placeholder);

        if (parts.length > 1) {
          if (parts[0]) elements.push(parts[0]);

          if (typeof value === "string") {
            elements.push(value);
          } else if (value) {
            elements.push(value);
          }

          template = parts.slice(1).join(placeholder);
        }
      }

      if (template) elements.push(template);

      return elements;
    } else {
      let result = translation.template;
      for (const param of translation.params) {
        const value = params[param];
        if (typeof value === "string") {
          result = result.replace(`{{${param}}}`, value);
        }
      }
      return result;
    }
  };

  const translateErrorMessage = (
    key: TranslationKey,
    error: unknown,
    params: Record<string, string> = {},
  ) => {
    if (!translations()) return key;
    return translateErrorMessageFn(translations()!, key, error, params);
  };

  const value = {
    translations,
    translate,
    translateErrorMessage,
  };

  return (
    <TranslationsContext.Provider value={value}>
      {props.children}
    </TranslationsContext.Provider>
  );
};

export const useTranslations = () => {
  const context = useContext(TranslationsContext);
  if (!context) {
    throw new Error("useTranslations: cannot find a TranslationsContext");
  }
  return context;
};
