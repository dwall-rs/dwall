// 职责：壁纸图片（本地文件）。path 为空时显示占位（可自定义）。
import { Show, type JSXElement } from "solid-js";
import { assetUrl } from "@/ipc";
import { clsx } from "@/utils";

interface Props {
  path: string | null;
  fit?: "cover" | "contain";
  class?: string;
  fallback?: JSXElement;
}

export function WallpaperImage(props: Props) {
  return (
    <Show
      when={props.path}
      fallback={
        props.fallback ?? (
          <div class={clsx("h-full w-full bg-card", props.class)} />
        )
      }
    >
      {(path) => (
        <img
          src={assetUrl(path())}
          alt=""
          class={clsx(
            "block h-full w-full",
            props.fit === "contain" ? "object-contain" : "object-cover",
            props.class,
          )}
        />
      )}
    </Show>
  );
}
