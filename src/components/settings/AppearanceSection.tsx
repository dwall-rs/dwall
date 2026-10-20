// Responsibility: Appearance & language group — three options + palette + title bar + its separator + language. Note: "follow the system" is the "System" option among the three.
import { SettingsGroup } from "./SettingsGroup";
import { SettingsRow } from "./SettingsRow";
import { ThemeSegmented } from "./ThemeSegmented";
import { LanguageRow } from "./LanguageRow";
import { Switch } from "@/components/ui/switch";
import { patch, save, settingsStore } from "@/store/settings.store";
import { setTitlebarBorder, uiStore } from "@/store/ui.store";
import { titleBarMatchesContent } from "@/store/ui.selectors";
import { t } from "@/i18n";
import { Sparkles } from "lucide-solid";
import { createSignal, Show } from "solid-js";

export function AppearanceSection() {
  const [flash, setFlash] = createSignal<string | null>(null);
  const ok = async (key: string) => {
    setFlash(key);
    await save();
    setFlash(null);
  };

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
        <SettingsRow
          label={t("settings.appearance.titleBar")}
          okShow={flash() === "titleBar"}
          desc={t("settings.appearance.titleBarDesc")}
          control={
            <Switch
              checked={
                settingsStore.config?.title_bar_color_follows_windows_theme
              }
              onCheckedChange={(checked) => {
                patch({ title_bar_color_follows_windows_theme: checked });
                void ok("titleBar");
              }}
            />
          }
        />
        {/* Only while the title bar has no colour of its own: once it follows the Windows
            theme, the two areas are visibly different and the switch would be decoration. */}
        <Show when={titleBarMatchesContent()}>
          <SettingsRow
            label={t("settings.appearance.contentBorder")}
            desc={t("settings.appearance.contentBorderDesc")}
            control={
              <Switch
                checked={uiStore.titlebarBorder}
                onCheckedChange={(checked) => setTitlebarBorder(checked)}
              />
            }
          />
        </Show>
        <SettingsRow
          label={t("settings.appearance.languageLabel")}
          desc={t("settings.appearance.languageDesc")}
          control={<LanguageRow />}
        />
      </div>
    </SettingsGroup>
  );
}
