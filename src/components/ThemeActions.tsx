import { Show } from "solid-js";
import { LazyButton, LazySpace } from "~/lazy";

interface ThemeActionsProps {
  themeExists: boolean;
  appliedThemeID?: string;
  currentThemeID: string;
  onDownload: () => void;
  onApply: () => void;
  onCloseTask: () => void;
  downloadThemeID?: string;
}

export const ThemeActions = (props: ThemeActionsProps) => {
  return (
    <LazySpace gap={8}>
      <Show
        when={props.themeExists}
        fallback={
          <LazyButton
            type="primary"
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
            <LazyButton onClick={props.onCloseTask} danger>
              停止
            </LazyButton>
          }
        >
          <LazyButton
            type="primary"
            disabled={!props.themeExists}
            onClick={props.onApply}
          >
            应用
          </LazyButton>
        </Show>
      </Show>
    </LazySpace>
  );
};
