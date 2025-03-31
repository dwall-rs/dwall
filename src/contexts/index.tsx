import type { ParentProps } from "solid-js";

import { TranslationsProvider } from "./TranslationsContext";
import { ConfigProvider } from "./ConfigContext";

export const AppProvider = (props: ParentProps) => {
  return (
    <TranslationsProvider>
      <ConfigProvider>{props.children}</ConfigProvider>
    </TranslationsProvider>
  );
};

export { useTranslations } from "./TranslationsContext";
export { useConfig } from "./ConfigContext";
