// 职责：外观与语言分组——三档 + 色板 + 语言。注意「自动跟随系统」即三档里的「系统」。
import { SettingsGroup } from "./SettingsGroup";
import { SettingsRow } from "./SettingsRow";
import { ThemeSegmented } from "./ThemeSegmented";
import { LanguageRow } from "./LanguageRow";
import { Sparkles } from "lucide-solid";

export function AppearanceSection() {
  return (
    <SettingsGroup
      icon={<Sparkles class="size-3.25" />}
      title="外观与语言"
      delay={20}
    >
      <div class="overflow-hidden rounded-[15px] border border-border bg-card shadow-[0_1px_2px_rgba(0,0,0,.05),0_8px_24px_rgba(0,0,0,.06)]">
        <SettingsRow
          label="外观 / Appearance"
          desc="选择亮色、暗色，或跟随系统。跟随系统会实时监听系统外观变化自动切换。"
          control={<ThemeSegmented />}
        />
        {/*<SemanticSwatches />*/}
        <SettingsRow
          label="语言 / Language"
          desc="界面语言，切换后立即生效。"
          control={<LanguageRow />}
        />
      </div>
    </SettingsGroup>
  );
}
