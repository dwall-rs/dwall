import { createEffect, createSignal, Show } from "solid-js";

import { useMonitor, useTask, useTheme } from "~/contexts";
import { Button } from "~/components/button";
import { validateTheme } from "~/commands";
import Download from "./Download";
import { t } from "~/i18n";

interface ThemeActionsProps {
  currentThemeID: string;
  themesDirectory?: string;
  isCustomized?: boolean;
}

const ThemeActions = (props: ThemeActionsProps) => {
  const theme = useTheme();
  const { id: monitorID } = useMonitor();

  const { handleTaskClosure } = useTask();

  const [spinning, setSpinning] = createSignal(false);
  const [dowloadingID, setDowloadingID] = createSignal<string>();

  const onApply = async () => {
    setSpinning(true);
    await theme.handleThemeApplication(monitorID, props.currentThemeID);
    setSpinning(false);
  };

  const onClose = () => {
    setSpinning(true);
    handleTaskClosure();
    setSpinning(false);
  };

  const [themeExists, setThemeExists] = createSignal(false);

  const checkThemeExists = async () => {
    if (!props.themesDirectory) {
      setThemeExists(false);
      return;
    }

    try {
      await validateTheme(
        props.themesDirectory,
        props.currentThemeID,
        props.isCustomized,
      );
      setThemeExists(true);
    } catch (e) {
      setThemeExists(false);
      console.error("Failed to check theme existence:", e);
    }
  };

  const handleDownload = () => {
    setDowloadingID(props.currentThemeID);
    theme.setDownloadingTheme(true);
  };

  createEffect(async () => {
    await checkThemeExists();
  });

  return (
    <Show
      when={themeExists()}
      fallback={
        <Show
          when={!dowloadingID()}
          fallback={
            <Download
              id={dowloadingID()!}
              onFinished={() => {
                setDowloadingID();
                checkThemeExists();
                theme.setDownloadingTheme(false);
              }}
            />
          }
        >
          <Button onClick={handleDownload} disabled={!!dowloadingID()}>
            {t("theme.button.download")}
          </Button>
        </Show>
      }
    >
      <Show
        when={theme.appliedThemeID() !== props.currentThemeID}
        fallback={
          <Button onClick={onClose} variant="destructive">
            {t("theme.button.stop")}
          </Button>
        }
      >
        <Button onClick={onApply} disabled={spinning() || !themeExists()}>
          {t("theme.button.apply")}
        </Button>
      </Show>
    </Show>
  );
};

export default ThemeActions;
