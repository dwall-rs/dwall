import { createSignal, Show } from "solid-js";
import { LazyButton, LazySpace } from "~/lazy";

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
            下载
          </LazyButton>
        }
      >
        <Show
          when={props.appliedThemeID !== props.currentThemeID}
          fallback={
            <LazyButton onClick={onClose} danger>
              停止
            </LazyButton>
          }
        >
          <LazyButton
            isLoading={spinning()}
            disabled={!props.themeExists}
            onClick={onApply}
          >
            应用
          </LazyButton>
        </Show>
      </Show>
    </LazySpace>
  );
};
