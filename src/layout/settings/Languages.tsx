import SettingsItem from "./SettingsItem";

import { t, setLocale, locale, LANGUAGES, type Locale } from "~/i18n";
import { Select } from "~/components/select";

const Languages = () => {
  return (
    <SettingsItem label={t("settings.label.language")}>
      <Select
        class="w-48"
        options={Object.keys(LANGUAGES).map((key) => ({
          value: key as Locale,
          label: LANGUAGES[key as Locale],
        }))}
        value={locale()}
        onChange={setLocale}
      />
    </SettingsItem>
  );
};

export default Languages;
