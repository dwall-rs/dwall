import { createMemo, createResource } from "solid-js";
import { getCustomizedThemes } from "~/commands";
import { useConfig } from "~/contexts";
import { themes } from "~/themes";

export const useThemes = () => {
  const { data: config } = useConfig();
  const [customizedThemes] = createResource(
    () => config()?.customized_themes_directory,
    getCustomizedThemes,
  );

  const allThemes = createMemo(() => [
    ...(customizedThemes() ?? []),
    ...themes,
  ]);

  return {
    allThemes,
  };
};
