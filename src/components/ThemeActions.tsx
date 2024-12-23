import { createSignal, Show } from "solid-js";
import { useAppContext } from "~/context";
import { LazyButton, LazySpace } from "~/lazy";
import { translate } from "~/utils/i18n";

export interface ThemeActionsProps {
  themeExists: boolean;
  appliedThemeID?: string;
  currentThemeID: string;
  onDownload: () => void;
  onApply: () => void;
  onCloseTask: () => void;
  downloadThemeID?: string;
}

export const ThemeActions = (props: ThemeActionsProps) => {
  const { translations } = useAppContext();

  const [spinning, setSpinning] = createSignal(false);

  const onApply = () => {
    setSpinning(true);
    props.onApply();
    setSpinning(false);
  };

  const onClose = () => {
    setSpinning(true);
    props.onCloseTask();
    setSpinning(false);
  };

  return (
    <LazySpace gap={8}>
      <Show
        when={props.themeExists}
        fallback={
          <LazyButton
            onClick={props.onDownload}
            disabled={!!props.downloadThemeID}
          >
            {translate(translations()!, "button-download")}
          </LazyButton>
        }
      >
        <Show
          when={props.appliedThemeID !== props.currentThemeID}
          fallback={
            <LazyButton onClick={onClose} danger>
              {translate(translations()!, "button-stop")}
            </LazyButton>
          }
        >
          <LazyButton
            isLoading={spinning()}
            disabled={!props.themeExists}
            onClick={onApply}
          >
            {translate(translations()!, "button-apply")}
          </LazyButton>
        </Show>
      </Show>
    </LazySpace>
  );
};
