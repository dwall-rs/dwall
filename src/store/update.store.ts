// 职责：应用更新——检查 / 下载安装 / 重启的状态机；网络配置复用 settings.store。

import { createStore } from "solid-js/store";

import { checkForUpdates, type Update, relaunchApp } from "@/ipc";
import { logger } from "@/utils";
import { settingsStore } from "./settings.store";

const log = logger.child("update");

export type UpdateStatus =
  | "idle"
  | "checking"
  | "latest"
  | "available"
  | "downloading"
  | "ready"
  | "error";

interface UpdateState {
  status: UpdateStatus;
  version: string | null;
  progress: number | null;
  error: string | null;
}

const [updateStore, setUpdateStore] = createStore<UpdateState>({
  status: "idle",
  version: null,
  progress: null,
  error: null,
});

/** 当前可安装的更新句柄；非响应式，派生字段已同步进 store。 */
let pending: Update | null = null;
let totalBytes = 0;
let receivedBytes = 0;

const check = async () => {
  if (updateStore.status === "checking" || updateStore.status === "downloading")
    return;

  setUpdateStore({
    status: "checking",
    version: null,
    progress: null,
    error: null,
  });

  try {
    const update = await checkForUpdates(
      settingsStore.config?.network ?? undefined,
    );
    if (!update) {
      setUpdateStore("status", "latest");
      return;
    }

    pending = update;
    setUpdateStore({ status: "available", version: update.version });
  } catch (e) {
    log.error("Failed to check for updates", e);
    setUpdateStore({ status: "error", error: String(e) });
  }
};

const install = async () => {
  if (!pending || updateStore.status !== "available") return;

  totalBytes = 0;
  receivedBytes = 0;
  setUpdateStore({ status: "downloading", progress: null, error: null });

  try {
    await pending.downloadAndInstall((event) => {
      if (event.event === "Started") {
        totalBytes = event.data.contentLength ?? 0;
        setUpdateStore("progress", totalBytes ? 0 : null);
      } else if (event.event === "Progress") {
        receivedBytes += event.data.chunkLength;
        setUpdateStore(
          "progress",
          totalBytes ? Math.min(receivedBytes / totalBytes, 1) : null,
        );
      } else {
        setUpdateStore("progress", 1);
      }
    });
    setUpdateStore("status", "ready");
    await relaunchApp();
  } catch (e) {
    log.error("Failed to install update", e);
    setUpdateStore({ status: "error", error: String(e) });
  }
};

const restart = () => relaunchApp();

export { updateStore, check, install, restart };
