// Responsibility: app updates — state machine for check / download-install / relaunch; network config reuses settings.store.

import { createStore } from "solid-js/store";

import { toast } from "@/components/ui/toast";
import { t } from "@/i18n";
import { checkForUpdates, type Update, relaunchApp } from "@/ipc";
import { logger } from "@/utils";
import { settingsStore } from "./settings.store";

const log = logger.child("update");

/** The element the start-up notification points at, set on the About group. */
const ABOUT_SECTION_ID = "settings-about";

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
  /** Release notes of the offered version, when the manifest carried them. */
  notes: string | null;
}

const [updateStore, setUpdateStore] = createStore<UpdateState>({
  status: "idle",
  version: null,
  progress: null,
  error: null,
  notes: null,
});

/** Handle of the update that can be installed right now; non-reactive — its derived fields are synced into the store. */
let pending: Update | null = null;
let totalBytes = 0;
let receivedBytes = 0;

/**
 * Releases the resource registered on the Rust side by `check_for_updates_cmd`.
 * Every check adds one entry to the resource table, so without this the handle
 * count only ever grows.
 */
const releasePending = () => {
  if (!pending) return;

  void pending.close().catch((e) => {
    log.warn("Failed to release the update resource", e);
  });
  pending = null;
};

/**
 * Tells the user there is a new version and takes them to where it is installed.
 *
 * The About group is always mounted, so the action only has to bring it into view; the
 * install itself stays a deliberate click there rather than a button in a notification
 * that would download and relaunch the app on a mis-click.
 */
const announceUpdate = (version: string) => {
  toast.info(t("settings.about.updateToastTitle", { version }), {
    description: t("settings.about.updateToastHint"),
    action: {
      label: t("settings.about.updateToastAction"),
      onClick: () => {
        globalThis.document
          ?.getElementById(ABOUT_SECTION_ID)
          ?.scrollIntoView({ behavior: "smooth", block: "center" });
      },
    },
  });
};

/**
 * Runs the check.
 *
 * `automatic` marks the one the window starts by itself. Such a check stays quiet when
 * it fails — nobody asked for it, and an "update check failed" card on every start with
 * no network is noise — and says so through a notification when it succeeds. A manual
 * check leaves the failure in the widget, where the user is waiting for an answer.
 */
const runCheck = async (automatic: boolean) => {
  if (updateStore.status === "checking" || updateStore.status === "downloading")
    return;

  releasePending();

  setUpdateStore({
    status: "checking",
    version: null,
    progress: null,
    error: null,
    notes: null,
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
    setUpdateStore({
      status: "available",
      version: update.version,
      notes: update.body ?? null,
    });

    if (automatic) announceUpdate(update.version);
  } catch (e) {
    log.error("Failed to check for updates", e);

    if (automatic) {
      // Back to the untouched state: the section shows its plain button again.
      setUpdateStore({ status: "idle", error: null });
      return;
    }

    setUpdateStore({ status: "error", error: String(e) });
  }
};

/** Checks on the user's request. */
const check = () => runCheck(false);

/** Whether this window has already checked by itself. */
let startupChecked = false;

/**
 * Checks once when the settings window opens, and announces a new version.
 *
 * The app calls this after loading the settings, because the check has to honour the
 * configured proxy. It runs once per window: the root component remounts on hot reload
 * and could otherwise ask the same question repeatedly.
 */
const checkOnStartup = async () => {
  if (startupChecked) return;

  startupChecked = true;
  await runCheck(true);
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
    releasePending();
    setUpdateStore({ status: "error", error: String(e) });
  }
};

const restart = () => relaunchApp();

export { updateStore, check, checkOnStartup, install, restart };
