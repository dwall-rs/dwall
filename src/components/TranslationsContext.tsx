import { createContext, useContext, type JSX } from "solid-js";
import { createResource } from "solid-js";
import { getTranslations } from "~/commands";
import {
  translate as translateFn,
  translateErrorMessage as translateErrorMessageFn,
} from "~/utils/i18n";

type TranslationsContextType = {
  translations: () => Translations | undefined;
  translate: (key: TranslationKey, params?: Record<string, string>) => string;
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
    params: Record<string, string> = {},
  ) => {
    if (!translations()) return key;
    return translateFn(translations()!, key, params);
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
