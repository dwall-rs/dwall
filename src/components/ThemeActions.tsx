import { createSignal, Show } from "solid-js";
import { LazyButton, LazySpace } from "~/lazy";
import { useMonitor, useTask, useTheme, useTranslations } from "~/contexts";

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
  const theme = useTheme();
  const { id: monitorID, list: monitors } = useMonitor();

  const { translate } = useTranslations();
  const { handleTaskClosure } = useTask();

  const [spinning, setSpinning] = createSignal(false);

  const onApply = () => {
    setSpinning(true);
    theme.handleThemeApplication(monitorID, monitors);
    setSpinning(false);
  };

  const onClose = () => {
    setSpinning(true);
    handleTaskClosure();
    setSpinning(false);
  };

  return (
    <LazySpace gap={8}>
      <Show
        when={theme.themeExists()}
        fallback={
          <Show when={!theme.downloadThemeID()}>
            <LazyButton
              onClick={() => theme.setDownloadThemeID(theme.currentTheme()!.id)}
              disabled={!!theme.downloadThemeID()}
            >
              {translate("button-download")}
            </LazyButton>
          </Show>
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
