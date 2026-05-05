import {
  createEffect,
  createMemo,
  createSignal,
  onMount,
  type ParentProps,
} from "solid-js";

import { AiOutlineDownload } from "solid-icons/ai";

import { message } from "@tauri-apps/plugin-dialog";
import { open } from "@tauri-apps/plugin-shell";

import { useUpdate } from "~/contexts";

import { toast } from "~/components/toast";
import { Button } from "~/components/button";
import { Progress } from "~/components/progress";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "~/components/dialog";

import { LazyMarkdown } from "~/lazy";
import { t } from "~/i18n";

const Updater = (props: ParentProps) => {
  const { update } = useUpdate();

  const [total, setTotal] = createSignal<number | undefined>();
  const [downloaded, setDownloaded] = createSignal<number | undefined>();
  const [error, setError] = createSignal<string | undefined>();

  const percentage = createMemo(() => {
    const totalValue = total() ?? 0;
    const downloadedValue = downloaded() ?? 0;
    return (totalValue === 0 ? 0 : downloadedValue / totalValue) * 100;
  });

  onMount(async () => {
    try {
      await update()!.download((event) => {
        switch (event.event) {
          case "Started":
            setTotal(event.data.contentLength ?? 0);
            break;
          case "Progress":
            setDownloaded((prev) => (prev ?? 0) + event.data.chunkLength);
            break;
          case "Finished":
            break;
        }
      });
    } catch (error) {
      const errorMessage = t("update.message.updateFailed", {
        error: String(error),
      });
      await message(errorMessage, {
        kind: "error",
      });
      setError(errorMessage);
    }
  });

  const updateErrorHelpMessage = (message: string) => {
    return (
      <div>
        <h4>{message}</h4>

        <div>
          {t("update.message.updateFailed")}
          <Button
            onClick={() =>
              open(
                (
                  update()!.rawJson.platforms as Record<
                    string,
                    Record<string, string>
                  >
                )["windows-x86_64"].url,
              )
            }
            icon={{ icon: <AiOutlineDownload />, ariaLabel: "Download" }}
            size="sm"
          />
        </div>
      </div>
    );
  };

  createEffect(() => {
    error() &&
      toast.error(updateErrorHelpMessage(error()!), {
        position: "bottom-right",
        duration: 5000,
      });
  });

  return (
    <Dialog>
      <DialogTrigger>{props.children}</DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {t("update.title.downloadingNewVersion", {
              version: update()!.version,
            })}
          </DialogTitle>
        </DialogHeader>

        <LazyMarkdown content={update()!.body ?? ""} />
        <Progress class="w-full" value={percentage()} />

        <DialogFooter>
          <Button
            onClick={() => update()!.install()}
            disabled={percentage() < 100}
          >
            {t("update.button.install")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default Updater;
