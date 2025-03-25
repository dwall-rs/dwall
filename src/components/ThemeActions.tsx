import { createSignal, Show } from "solid-js";
import { LazyButton, LazySpace } from "~/lazy";
import { useTranslations } from "./TranslationsContext";
import { useAppContext } from "~/context";

export interface ThemeActionsProps {
  themeExists: boolean;
  appliedThemeID?: string;
  currentThemeID: string;
  onDownload: () => void;
  onApply: () => void;
  onCloseTask: () => void;
  downloadThemeID?: string;
}

export const ThemeActions = () => {
  const { theme, task } = useAppContext();

  const { translate } = useTranslations();

  const [spinning, setSpinning] = createSignal(false);

  const onApply = () => {
    setSpinning(true);
    theme.handleThemeApplication();
    setSpinning(false);
  };

  const onClose = () => {
    setSpinning(true);
    task.handleClosure();
    setSpinning(false);
  };

  return (
    <LazySpace gap={8}>
      <Show
        when={theme.themeExists()}
        fallback={
          <LazyButton
            onClick={() => theme.setDownloadThemeID(theme.currentTheme()!.id)}
            disabled={!!theme.downloadThemeID()}
          >
            {translate("button-download")}
          </LazyButton>
        }
      >
        <Show
          when={theme.appliedThemeID() !== theme.currentTheme()!.id}
          fallback={
            <LazyButton onClick={onClose} appearance="danger">
              {translate("button-stop")}
            </LazyButton>
          }
        >
          <LazyButton
            isLoading={spinning()}
            disabled={!theme.themeExists()}
            onClick={onApply}
          >
            {translate("button-apply")}
          </LazyButton>
        </Show>
      </Show>
    </LazySpace>
  );
};
