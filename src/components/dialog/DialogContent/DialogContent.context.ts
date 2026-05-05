import { createContext, useContext, type Setter } from "solid-js";

interface DialogContentContextValue {
  setTitleID: Setter<string | undefined>;
  setDescriptionID: Setter<string | undefined>;
}

export const DialogContentContext = createContext<DialogContentContextValue>();

export const useDialogContentContext = () => {
  const ctx = useContext(DialogContentContext);
  if (!ctx)
    throw new Error(
      "useDialogContentContext must be used within an DialogContentProvider",
    );

  return ctx;
};
