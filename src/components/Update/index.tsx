import { Show } from "solid-js";
import UpdateDialog from "./UpdateDialog";
import { useUpdate } from "~/contexts";

const Updater = () => {
  const { update, showUpdateDialog } = useUpdate();

  return (
    <Show when={showUpdateDialog()}>
      <UpdateDialog update={update()!} />
    </Show>
  );
};

export default Updater;
