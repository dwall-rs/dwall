import {
  type Accessor,
  createContext,
  type Setter,
  useContext,
} from "solid-js";

interface DialogContextValue {
  show: Accessor<boolean>;
  setShow: Setter<boolean>;
  open: Accessor<boolean>;

  setOpen: (open: boolean) => void;
}

export const DialogContext = createContext<DialogContextValue>();

export const useDialogContext = () => {
  const context = useContext(DialogContext);
  if (!context) {
    throw new Error("useDialogContext 必须用在 <Dialog> 内部");
  }
  return context;
};
