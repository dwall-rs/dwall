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
  setOpen: Setter<boolean>;
}

export const DialogContext = createContext<DialogContextValue>();

export const useDialogContext = () => {
  const context = useContext(DialogContext);
  if (!context) {
    throw new Error("useDialogContext must be used within a DialogProvider");
  }
  return context;
};
