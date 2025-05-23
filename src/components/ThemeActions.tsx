import { createSignal, Show } from "solid-js";

import { LazyButton } from "~/lazy";
import DangerButton from "./DangerButton";

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
  const { id: monitorID } = useMonitor();

  const { translate } = useTranslations();
  const { handleTaskClosure } = useTask();

  const [spinning, setSpinning] = createSignal(false);

  const onApply = async () => {
    setSpinning(true);
    await theme.handleThemeApplication(monitorID);
    setSpinning(false);
  };

  const onClose = () => {
    setSpinning(true);
    handleTaskClosure();
    setSpinning(false);
  };

  return (
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
          <DangerButton onClick={onClose}>
            {translate("button-stop")}
          </DangerButton>
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
  );
};
