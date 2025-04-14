import { createContext, type ParentProps, useContext } from "solid-js";
import { useUpdateManager } from "~/hooks/useUpdateManager";

interface UpdateContext {
  showUpdateDialog: Accessor<boolean | undefined>;
  setShowUpdateDialog: Setter<boolean | undefined>;
  update: Resource<Update | null | undefined>;
  recheckUpdate: Refetcher<Update>;
}

const UpdateContext = createContext<UpdateContext>();

export const UpdateProvider = (props: ParentProps) => {
  const update = useUpdateManager();

  return (
    <UpdateContext.Provider value={update}>
      {props.children}
    </UpdateContext.Provider>
  );
};

export const useUpdate = () => {
  const context = useContext(UpdateContext);
  if (!context) {
    throw new Error("useUpdate: 必须在UpdateProvider内部使用");
  }
  return context;
};
