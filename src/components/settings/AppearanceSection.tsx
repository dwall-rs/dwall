// 职责：外观与语言分组——三档 + 色板 + 语言。注意「自动跟随系统」即三档里的「系统」。
import { SettingsGroup } from "./SettingsGroup";
import { SettingsRow } from "./SettingsRow";
import { ThemeSegmented } from "./ThemeSegmented";
import { LanguageRow } from "./LanguageRow";
import { t } from "@/i18n";
import { Sparkles } from "lucide-solid";

export function AppearanceSection() {
  return (
    <SettingsGroup
      icon={<Sparkles class="size-3.25" />}
      title={t("settings.appearance.title")}
      delay={20}
    >
      <div class="overflow-hidden rounded-[15px] border border-border bg-card shadow-[0_1px_2px_rgba(0,0,0,.05),0_8px_24px_rgba(0,0,0,.06)]">
        <SettingsRow
          label={t("settings.appearance.appearanceLabel")}
          desc={t("settings.appearance.appearanceDesc")}
          control={<ThemeSegmented />}
        />
        {/*<SemanticSwatches />*/}
        <SettingsRow
          label={t("settings.appearance.languageLabel")}
          desc={t("settings.appearance.languageDesc")}
          control={<LanguageRow />}
        />
      </div>
    </SettingsGroup>
  );
}
