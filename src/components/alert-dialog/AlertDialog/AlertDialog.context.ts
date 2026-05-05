import {
  type Accessor,
  createContext,
  type Setter,
  useContext,
} from "solid-js";

interface AlertDialogContextValue {
  show: Accessor<boolean>;
  setShow: Setter<boolean>;
  open: Accessor<boolean>;
  setOpen: Setter<boolean>;
}

export const AlertDialogContext = createContext<AlertDialogContextValue>();

export const useAlertDialogContext = () => {
  const context = useContext(AlertDialogContext);
  if (!context) {
    throw new Error(
      "useAlertDialogContext must be used within an AlertDialogProvider",
    );
  }
  return context;
};
