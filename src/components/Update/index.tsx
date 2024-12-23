import { Show } from "solid-js";
import { useAppContext } from "~/context";
import UpdateDialog from "./UpdateDialog";

const Updater = () => {
  const {
    update: { resource, showDialog },
  } = useAppContext();

  return (
    <Show when={showDialog()}>
      <UpdateDialog update={resource()!} />
    </Show>
  );
};

export default Updater;
