import { convertFileSrc } from "@tauri-apps/api/core";

/** Convert an absolute file path into a URL the webview can load. */
export const assetUrl = (path: string) => convertFileSrc(path);
